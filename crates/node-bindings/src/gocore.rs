//! Utilities for launching a go-ethereum dev-mode instance.

use crate::unused_port;
use atoms_genesis::{CliqueConfig, Genesis};
use base_primitives::{hex, IcanAddress, B256};
use libgoldilocks::SigningKey;
use std::{
    borrow::Cow,
    fs::{create_dir, File},
    io::{BufRead, BufReader},
    net::SocketAddr,
    path::PathBuf,
    process::{Child, ChildStderr, Command, Stdio},
    time::{Duration, Instant},
};
use tempfile::tempdir;
use thiserror::Error;
use url::Url;

/// How long we will wait for gocore to indicate that it is ready.
const GOCORE_STARTUP_TIMEOUT: Duration = Duration::from_secs(20);

/// Timeout for waiting for gocore to add a peer.
const GOCORE_DIAL_LOOP_TIMEOUT: Duration = Duration::from_secs(30);

/// The exposed APIs
const API: &str = "xcb,net,web3,txpool,admin,personal,miner,debug";

/// The gocore command
const GOCORE: &str = "gocore";

/// Errors that can occur when working with the [`GocoreInstance`].
#[derive(Debug)]
pub enum GocoreInstanceError {
    /// Timed out waiting for a message from gocore's stderr.
    Timeout(String),

    /// A line could not be read from the gocore stderr.
    ReadLineError(std::io::Error),

    /// The child gocore process's stderr was not captured.
    NoStderr,
}

/// A gocore instance. Will close the instance when dropped.
///
/// Construct this using [`Gocore`].
#[derive(Debug)]
pub struct GocoreInstance {
    pid: Child,
    port: u16,
    ipc: Option<PathBuf>,
    data_dir: Option<PathBuf>,
    p2p_port: Option<u16>,
    genesis: Option<Genesis>,
    clique_private_key: Option<SigningKey>,
}

impl GocoreInstance {
    /// Returns the port of this instance
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns the p2p port of this instance
    pub fn p2p_port(&self) -> Option<u16> {
        self.p2p_port
    }

    /// Returns the HTTP endpoint of this instance
    pub fn endpoint(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    /// Returns the Websocket endpoint of this instance
    pub fn ws_endpoint(&self) -> String {
        format!("ws://localhost:{}", self.port)
    }

    /// Returns the IPC endpoint of this instance
    pub fn ipc_endpoint(&self) -> String {
        if let Some(ipc) = self.ipc.clone() {
            ipc.display().to_string()
        } else {
            "gocore.ipc".to_string()
        }
    }

    /// Returns the HTTP endpoint url of this instance
    pub fn endpoint_url(&self) -> Url {
        Url::parse(&self.endpoint()).unwrap()
    }

    /// Returns the Websocket endpoint url of this instance
    pub fn ws_endpoint_url(&self) -> Url {
        Url::parse(&self.ws_endpoint()).unwrap()
    }

    /// Returns the path to this instances' data directory
    pub fn data_dir(&self) -> &Option<PathBuf> {
        &self.data_dir
    }

    /// Returns the genesis configuration used to configure this instance
    pub fn genesis(&self) -> &Option<Genesis> {
        &self.genesis
    }

    /// Returns the private key used to configure clique on this instance
    #[deprecated = "clique support was removed in gocore >=1.14"]
    pub fn clique_private_key(&self) -> &Option<SigningKey> {
        &self.clique_private_key
    }

    /// Takes the stderr contained in the child process.
    ///
    /// This leaves a `None` in its place, so calling methods that require a stderr to be present
    /// will fail if called after this.
    pub fn stderr(&mut self) -> Result<ChildStderr, GocoreInstanceError> {
        self.pid.stderr.take().ok_or(GocoreInstanceError::NoStderr)
    }

    /// Blocks until gocore adds the specified peer, using 20s as the timeout.
    ///
    /// Requires the stderr to be present in the `GocoreInstance`.
    pub fn wait_to_add_peer(&mut self, id: B256) -> Result<(), GocoreInstanceError> {
        let mut stderr = self.pid.stderr.as_mut().ok_or(GocoreInstanceError::NoStderr)?;
        let mut err_reader = BufReader::new(&mut stderr);
        let mut line = String::new();
        let start = Instant::now();

        while start.elapsed() < GOCORE_DIAL_LOOP_TIMEOUT {
            line.clear();
            err_reader.read_line(&mut line).map_err(GocoreInstanceError::ReadLineError)?;

            // gocore ids are trunated
            let truncated_id = hex::encode(&id.0[..8]);
            if line.contains("Adding p2p peer") && line.contains(&truncated_id) {
                return Ok(());
            }
        }
        Err(GocoreInstanceError::Timeout("Timed out waiting for gocore to add a peer".into()))
    }
}

impl Drop for GocoreInstance {
    fn drop(&mut self) {
        self.pid.kill().expect("could not kill gocore");
    }
}

/// Whether or not gocore is in `dev` mode and configuration options that depend on the mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GocoreMode {
    /// Options that can be set in dev mode
    Dev(DevOptions),
    /// Options that cannot be set in dev mode
    NonDev(PrivateNetOptions),
}

impl Default for GocoreMode {
    fn default() -> Self {
        Self::Dev(Default::default())
    }
}

/// Configuration options that can be set in dev mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DevOptions {
    /// The interval at which the dev network will mine new blocks.
    pub block_time: Option<u64>,
}

/// Configuration options that cannot be set in dev mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrivateNetOptions {
    /// The p2p port to use.
    pub p2p_port: Option<u16>,

    /// Whether or not peer discovery is enabled.
    pub discovery: bool,
}

impl Default for PrivateNetOptions {
    fn default() -> Self {
        Self { p2p_port: None, discovery: true }
    }
}

/// Errors that can occur when working with the [`Gocore`].
#[derive(Debug, Error)]
pub enum GocoreError {
    /// Clique private key error
    #[error("clique address error: {0}")]
    CliqueAddressError(String),
    /// The network id was not set.
    #[error("the network ID was not set")]
    NetworkIdNotSet,
    /// Could not create the data directory.
    #[error("could not create directory: {0}")]
    CreateDirError(std::io::Error),
    /// No stderr was captured from the child process.
    #[error("no stderr was captured from the process")]
    NoStderr,
    /// Timed out waiting for gocore to start.
    #[error("timed out waiting for gocore to spawn; is gocore installed?")]
    Timeout,
    /// Encountered a fatal error.
    #[error("fatal error: {0}")]
    Fatal(String),
    /// A line could not be read from the gocore stderr.
    #[error("could not read line from gocore stderr: {0}")]
    ReadLineError(std::io::Error),
    /// Genesis error
    #[error("genesis error occurred: {0}")]
    GenesisError(String),
    /// Gocore init error
    #[error("gocore init error occurred")]
    InitError,
    /// Spawn gocore error
    #[error("could not spawn gocore: {0}")]
    SpawnError(std::io::Error),
    /// Wait error
    #[error("could not wait for gocore to exit: {0}")]
    WaitError(std::io::Error),
}

/// Builder for launching `gocore`.
///
/// # Panics
///
/// If `spawn` is called without `gocore` being available in the user's $PATH
///
/// # Example
///
/// ```no_run
/// use atoms_node_bindings::Gocore;
///
/// let port = 8545u16;
/// let url = format!("http://localhost:{}", port).to_string();
///
/// let gocore = Gocore::new().port(port).block_time(5000u64).spawn();
///
/// drop(gocore); // this will kill the instance
/// ```
#[derive(Clone, Debug, Default)]
#[must_use = "This Builder struct does nothing unless it is `spawn`ed"]
pub struct Gocore {
    program: Option<PathBuf>,
    port: Option<u16>,
    authrpc_port: Option<u16>,
    ipc_path: Option<PathBuf>,
    ipc_enabled: bool,
    data_dir: Option<PathBuf>,
    network_id: Option<u64>,
    insecure_unlock: bool,
    genesis: Option<Genesis>,
    mode: GocoreMode,
    clique_private_key: Option<SigningKey>,
}

impl Gocore {
    /// Creates an empty Gocore builder.
    ///
    /// The mnemonic is chosen randomly.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a Gocore builder which will execute `gocore` at the given path.
    ///
    /// # Example
    ///
    /// ```
    /// use atoms_node_bindings::Gocore;
    /// # fn a() {
    /// let gocore = Gocore::at("../go-core/build/bin/gocore").spawn();
    ///
    /// println!("Gocore running at `{}`", gocore.endpoint());
    /// # }
    /// ```
    pub fn at(path: impl Into<PathBuf>) -> Self {
        Self::new().path(path)
    }

    /// Returns whether the node is launched in Clique consensus mode.
    pub fn is_clique(&self) -> bool {
        self.clique_private_key.is_some()
    }

    /// Calculates the address of the Clique consensus address.
    pub fn clique_address(&self) -> Option<IcanAddress> {
        self.clique_private_key
            .as_ref()
            .map(|pk| IcanAddress::from_public_key(pk.verifying_key(), self.network_id.unwrap()))
    }

    /// Sets the `path` to the `gocore` executable
    ///
    /// By default, it's expected that `gocore` is in `$PATH`, see also
    /// [`std::process::Command::new()`]
    pub fn path<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.program = Some(path.into());
        self
    }

    /// Sets the Clique Private Key to the `gocore` executable, which will be later
    /// loaded on the node.
    ///
    /// The address derived from this private key will be used to set the `miner.etherbase` field
    /// on the node.
    #[deprecated = "clique support was removed in gocore >=1.14"]
    pub fn set_clique_private_key<T: Into<SigningKey>>(mut self, private_key: T) -> Self {
        self.clique_private_key = Some(private_key.into());
        self
    }

    /// Sets the port which will be used when the `gocore-cli` instance is launched.
    ///
    /// If port is 0 then the OS will choose a random port.
    /// [GocoreInstance::port] will return the port that was chosen.
    pub fn port<T: Into<u16>>(mut self, port: T) -> Self {
        self.port = Some(port.into());
        self
    }

    /// Sets the port which will be used for incoming p2p connections.
    ///
    /// This will put the gocore instance into non-dev mode, discarding any previously set dev-mode
    /// options.
    pub fn p2p_port(mut self, port: u16) -> Self {
        match &mut self.mode {
            GocoreMode::Dev(_) => {
                self.mode = GocoreMode::NonDev(PrivateNetOptions {
                    p2p_port: Some(port),
                    ..Default::default()
                })
            }
            GocoreMode::NonDev(opts) => opts.p2p_port = Some(port),
        }
        self
    }

    /// Sets the block-time which will be used when the `gocore-cli` instance is launched.
    ///
    /// This will put the gocore instance in `dev` mode, discarding any previously set options that
    /// cannot be used in dev mode.
    pub fn block_time(mut self, block_time: u64) -> Self {
        self.mode = GocoreMode::Dev(DevOptions { block_time: Some(block_time) });
        self
    }

    /// Sets the network id for the gocore instance.
    pub fn network_id(mut self, network_id: u64) -> Self {
        self.network_id = Some(network_id);
        self
    }

    /// Allow gocore to unlock accounts when rpc apis are open.
    pub fn insecure_unlock(mut self) -> Self {
        self.insecure_unlock = true;
        self
    }

    /// Enable IPC for the gocore instance.
    pub fn enable_ipc(mut self) -> Self {
        self.ipc_enabled = true;
        self
    }

    /// Disable discovery for the gocore instance.
    ///
    /// This will put the gocore instance into non-dev mode, discarding any previously set dev-mode
    /// options.
    pub fn disable_discovery(mut self) -> Self {
        self.inner_disable_discovery();
        self
    }

    fn inner_disable_discovery(&mut self) {
        match &mut self.mode {
            GocoreMode::Dev(_) => {
                self.mode =
                    GocoreMode::NonDev(PrivateNetOptions { discovery: false, ..Default::default() })
            }
            GocoreMode::NonDev(opts) => opts.discovery = false,
        }
    }

    /// Sets the IPC path for the socket.
    pub fn ipc_path<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.ipc_path = Some(path.into());
        self
    }

    /// Sets the data directory for gocore.
    pub fn data_dir<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    /// Sets the `genesis.json` for the gocore instance.
    ///
    /// If this is set, gocore will be initialized with `gocore init` and the `--datadir` option will be
    /// set to the same value as `data_dir`.
    ///
    /// This is destructive and will overwrite any existing data in the data directory.
    pub fn genesis(mut self, genesis: Genesis) -> Self {
        self.genesis = Some(genesis);
        self
    }

    /// Sets the port for authenticated RPC connections.
    pub fn authrpc_port(mut self, port: u16) -> Self {
        self.authrpc_port = Some(port);
        self
    }

    /// Consumes the builder and spawns `gocore`.
    ///
    /// # Panics
    ///
    /// If spawning the instance fails at any point.
    #[track_caller]
    pub fn spawn(self) -> GocoreInstance {
        self.try_spawn().unwrap()
    }

    /// Consumes the builder and spawns `gocore`. If spawning fails, returns an error.
    pub fn try_spawn(mut self) -> Result<GocoreInstance, GocoreError> {
        let bin_path = match self.program.as_ref() {
            Some(bin) => bin.as_os_str(),
            None => GOCORE.as_ref(),
        }
        .to_os_string();
        let mut cmd = Command::new(&bin_path);
        // gocore uses stderr for its logs
        cmd.stderr(Stdio::piped());

        // If no port provided, let the os chose it for us
        let mut port = self.port.unwrap_or(0);
        let port_s = port.to_string();

        // If IPC is not enabled on the builder, disable it.
        if !self.ipc_enabled {
            cmd.arg("--ipcdisable");
        }

        // Open the HTTP API
        cmd.arg("--http");
        cmd.arg("--http.port").arg(&port_s);
        cmd.arg("--http.api").arg(API);

        // Open the WS API
        cmd.arg("--ws");
        cmd.arg("--ws.port").arg(port_s);
        cmd.arg("--ws.api").arg(API);

        // pass insecure unlock flag if set
        let is_clique = self.is_clique();
        if self.insecure_unlock || is_clique {
            cmd.arg("--allow-insecure-unlock");
        }

        if is_clique {
            self.inner_disable_discovery();
        }

        // Gocore doesn't support authrpc

        // // Set the port for authenticated APIs
        // let authrpc_port = self.authrpc_port.unwrap_or_else(&mut unused_port);
        // cmd.arg("--authrpc.port").arg(authrpc_port.to_string());

        // use gocore init to initialize the datadir if the genesis exists
        if is_clique {
            let clique_addr = self.clique_address();
            if let Some(genesis) = &mut self.genesis {
                // set up a clique config with an instant sealing period and short (8 block) epoch
                let clique_config = CliqueConfig { period: Some(0), epoch: Some(8) };
                genesis.config.clique = Some(clique_config);

                let clique_addr = clique_addr.ok_or(GocoreError::CliqueAddressError(
                    "could not calculates the address of the Clique consensus address.".to_string(),
                ))?;

                // set the extraData field
                let extra_data_bytes =
                    [&[0u8; 32][..], clique_addr.as_ref(), &[0u8; 65][..]].concat();
                genesis.extra_data = extra_data_bytes.into();

                // we must set the corebase if using clique
                // need to use format! / Debug here because the Address Display impl doesn't show
                // the entire address
                cmd.arg("--miner.corebase").arg(format!("{clique_addr:?}"));
            }

            let clique_addr = self.clique_address().ok_or(GocoreError::CliqueAddressError(
                "could not calculates the address of the Clique consensus address.".to_string(),
            ))?;

            self.genesis = Some(Genesis::clique_genesis(
                self.network_id.ok_or(GocoreError::NetworkIdNotSet)?,
                clique_addr,
            ));

            // we must set the corebase if using clique
            // need to use format! / Debug here because the Address Display impl doesn't show the
            // entire address
            cmd.arg("--miner.corebase").arg(format!("{clique_addr:?}"));
        }

        if let Some(genesis) = &self.genesis {
            // create a temp dir to store the genesis file
            let temp_genesis_dir_path = tempdir().map_err(GocoreError::CreateDirError)?.into_path();

            // create a temp dir to store the genesis file
            let temp_genesis_path = temp_genesis_dir_path.join("genesis.json");

            // create the genesis file
            let mut file = File::create(&temp_genesis_path).map_err(|_| {
                GocoreError::GenesisError("could not create genesis file".to_string())
            })?;

            // serialize genesis and write to file
            serde_json::to_writer_pretty(&mut file, &genesis).map_err(|_| {
                GocoreError::GenesisError("could not write genesis to file".to_string())
            })?;

            let mut init_cmd = Command::new(bin_path);
            if let Some(data_dir) = &self.data_dir {
                init_cmd.arg("--datadir").arg(data_dir);
            }

            // set the stderr to null so we don't pollute the test output
            init_cmd.stderr(Stdio::null());

            init_cmd.arg("init").arg(temp_genesis_path);
            let res = init_cmd
                .spawn()
                .map_err(GocoreError::SpawnError)?
                .wait()
                .map_err(GocoreError::WaitError)?;
            // .expect("failed to wait for gocore init to exit");
            if !res.success() {
                return Err(GocoreError::InitError);
            }

            // clean up the temp dir which is now persisted
            std::fs::remove_dir_all(temp_genesis_dir_path).map_err(|_| {
                GocoreError::GenesisError("could not remove genesis temp dir".to_string())
            })?;
        }

        if let Some(data_dir) = &self.data_dir {
            cmd.arg("--datadir").arg(data_dir);

            // create the directory if it doesn't exist
            if !data_dir.exists() {
                create_dir(data_dir).map_err(GocoreError::CreateDirError)?;
            }
        }

        // Dev mode with custom block time
        let mut p2p_port = match self.mode {
            GocoreMode::Dev(DevOptions { block_time }) => {
                cmd.arg("--dev");
                if let Some(block_time) = block_time {
                    cmd.arg("--dev.period").arg(block_time.to_string());
                }
                None
            }
            GocoreMode::NonDev(PrivateNetOptions { p2p_port, discovery }) => {
                // if no port provided, let the os chose it for us
                let port = p2p_port.unwrap_or(0);
                cmd.arg("--port").arg(port.to_string());

                // disable discovery if the flag is set
                if !discovery {
                    cmd.arg("--nodiscover");
                }
                Some(port)
            }
        };

        if let Some(network_id) = self.network_id {
            cmd.arg("--networkid").arg(network_id.to_string());
        }

        // debug verbosity is needed to check when peers are added
        cmd.arg("--verbosity").arg("4");

        if let Some(ipc) = &self.ipc_path {
            cmd.arg("--ipcpath").arg(ipc);
        }

        let mut child = cmd.spawn().map_err(GocoreError::SpawnError)?;

        let stderr = child.stderr.ok_or(GocoreError::NoStderr)?;

        let start = Instant::now();
        let mut reader = BufReader::new(stderr);

        // we shouldn't need to wait for p2p to start if gocore is in dev mode - p2p is disabled in
        // dev mode
        let mut p2p_started = matches!(self.mode, GocoreMode::Dev(_));
        let mut http_started = false;

        loop {
            if start + GOCORE_STARTUP_TIMEOUT <= Instant::now() {
                return Err(GocoreError::Timeout);
            }

            let mut line = String::with_capacity(120);
            reader.read_line(&mut line).map_err(GocoreError::ReadLineError)?;

            if matches!(self.mode, GocoreMode::NonDev(_)) && line.contains("Started P2P networking")
            {
                p2p_started = true;
            }

            if !matches!(self.mode, GocoreMode::Dev(_)) {
                // try to find the p2p port, if not in dev mode
                if line.contains("New local node record") {
                    if let Some(port) = extract_value("tcp=", &line) {
                        p2p_port = port.parse::<u16>().ok();
                    }
                }
            }

            // gocore 1.9.23 uses "server started" while 1.9.18 uses "endpoint opened"
            // the unauthenticated api is used for regular non-engine API requests
            if line.contains("HTTP endpoint opened")
                || (line.contains("HTTP server started") && !line.contains("auth=true"))
            {
                // Extracts the address from the output
                if let Some(addr) = extract_endpoint(&line) {
                    // use the actual http port
                    port = addr.port();
                }

                http_started = true;
            }

            // Encountered an error such as Fatal: Error starting protocol stack: listen tcp
            // 127.0.0.1:8545: bind: address already in use
            if line.contains("Fatal:") {
                return Err(GocoreError::Fatal(line));
            }

            if p2p_started && http_started {
                break;
            }
        }

        child.stderr = Some(reader.into_inner());

        Ok(GocoreInstance {
            pid: child,
            port,
            ipc: self.ipc_path,
            data_dir: self.data_dir,
            p2p_port,
            genesis: self.genesis,
            clique_private_key: self.clique_private_key,
        })
    }
}

// extracts the value for the given key and line
fn extract_value<'a>(key: &str, line: &'a str) -> Option<&'a str> {
    let mut key = Cow::from(key);
    if !key.ends_with('=') {
        key = Cow::from(format!("{}=", key));
    }
    line.find(key.as_ref()).map(|pos| {
        let start = pos + key.len();
        let end = line[start..].find(' ').map(|i| start + i).unwrap_or(line.len());
        line[start..end].trim()
    })
}

// extracts the value for the given key and line
fn extract_endpoint(line: &str) -> Option<SocketAddr> {
    let val = extract_value("endpoint=", line)?;
    val.parse::<SocketAddr>().ok()
}

// These tests should use a different datadir for each `Gocore` spawned
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_extract_address() {
        let line = "INFO [07-01|13:20:42.774] HTTP server started                      endpoint=127.0.0.1:8545 auth=false prefix= cors= vhosts=localhost";
        assert_eq!(extract_endpoint(line), Some(SocketAddr::from(([127, 0, 0, 1], 8545))));
    }

    #[test]
    fn port_0() {
        run_with_tempdir(|_| {
            let _gocore = Gocore::new().disable_discovery().port(0u16).spawn();
        });
    }

    /// Allows running tests with a temporary directory, which is cleaned up after the function is
    /// called.
    ///
    /// Helps with tests that spawn a helper instance, which has to be dropped before the temporary
    /// directory is cleaned up.
    #[track_caller]
    fn run_with_tempdir(f: impl Fn(&Path)) {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path();
        f(temp_dir_path);
        #[cfg(not(windows))]
        temp_dir.close().unwrap();
    }

    #[test]
    fn p2p_port() {
        run_with_tempdir(|temp_dir_path| {
            let gocore = Gocore::new().disable_discovery().data_dir(temp_dir_path).spawn();
            let p2p_port = gocore.p2p_port();
            assert!(p2p_port.is_some());
        });
    }

    #[test]
    fn explicit_p2p_port() {
        run_with_tempdir(|temp_dir_path| {
            // if a p2p port is explicitly set, it should be used
            let gocore = Gocore::new().p2p_port(1234).data_dir(temp_dir_path).spawn();
            let p2p_port = gocore.p2p_port();
            assert_eq!(p2p_port, Some(1234));
        });
    }

    #[test]
    fn dev_mode() {
        run_with_tempdir(|temp_dir_path| {
            // dev mode should not have a p2p port, and dev should be the default
            let gocore = Gocore::new().data_dir(temp_dir_path).spawn();
            let p2p_port = gocore.p2p_port();
            assert!(p2p_port.is_none(), "{p2p_port:?}");
        })
    }

    #[test]
    #[ignore = "fails on gocore >=1.14"]
    #[allow(deprecated)]
    fn clique_correctly_configured() {
        run_with_tempdir(|temp_dir_path| {
            let private_key = SigningKey::random(&mut rand::thread_rng());
            let gocore = Gocore::new()
                .set_clique_private_key(private_key)
                .network_id(1337u64)
                .data_dir(temp_dir_path)
                .spawn();

            assert!(gocore.p2p_port.is_some());
            assert!(gocore.clique_private_key().is_some());
            assert!(gocore.genesis().is_some());
        })
    }
}
