//! Example of using `MnemonicBuilder` to access a wallet from a mnemonic phrase.

use alloy_signer::{coins_bip39::English, MnemonicBuilder};

// NOTE: `rand` is not exported anymore, now private.
use rand;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let phrase = "work man father plunge mystery proud hollow address reunion sauce theory bonus";
    let index = 0u32;
    let password = "TREZOR123";

    // Access mnemonic phrase with password
    // Child key at derivation path: m/44'/60'/0'/0/{index}
    let wallet = MnemonicBuilder::<English>::default()
        .phrase(phrase)
        .index(index)?
        // Use this if your mnemonic is encrypted
        .password(password)
        .build()?;

    println!("Wallet: {:?}", wallet);

    // Generate a random wallet (24 word phrase) at custom derivation path
    let mut rng = rand::thread_rng();
    let wallet = MnemonicBuilder::<English>::default()
        .word_count(24)
        .derivation_path("m/44'/60'/0'/2/1")?
        // Optionally add this if you want the generated mnemonic to be written
        // to a file
        // .write_to(path)
        .build_random(&mut rng)?;

    println!("Random wallet: {:?}", wallet);

    Ok(())
}
