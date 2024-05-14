(function() {var type_impls = {
"alloy":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-RpcError%3CE,+ErrResp%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#5\">source</a><a href=\"#impl-Debug-for-RpcError%3CE,+ErrResp%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;E, ErrResp&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;<div class=\"where\">where\n    E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,\n    ErrResp: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#5\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","alloy::transports::TransportError"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Display-for-RpcError%3CE,+ErrResp%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#5\">source</a><a href=\"#impl-Display-for-RpcError%3CE,+ErrResp%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;E, ErrResp&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a> for <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;<div class=\"where\">where\n    <a class=\"struct\" href=\"alloy/rpc/json_rpc/struct.ErrorPayload.html\" title=\"struct alloy::rpc::json_rpc::ErrorPayload\">ErrorPayload</a>&lt;ErrResp&gt;: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a>,\n    E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#5\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, __formatter: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html#tymethod.fmt\">Read more</a></div></details></div></details>","Display","alloy::transports::TransportError"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Error-for-RpcError%3CE,+ErrResp%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#5\">source</a><a href=\"#impl-Error-for-RpcError%3CE,+ErrResp%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;E, ErrResp&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> for <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;<div class=\"where\">where\n    E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>,\n    <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.source\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#5\">source</a><a href=\"#method.source\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html#method.source\" class=\"fn\">source</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;(dyn <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + 'static)&gt;</h4></section></summary><div class='docblock'>The lower-level source of this error, if any. <a href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html#method.source\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.description\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/error.rs.html#110\">source</a></span><a href=\"#method.description\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html#method.description\" class=\"fn\">description</a>(&amp;self) -&gt; &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a></h4></section></summary><span class=\"item-info\"><div class=\"stab deprecated\"><span class=\"emoji\">👎</span><span>Deprecated since 1.42.0: use the Display impl or to_string()</span></div></span><div class='docblock'> <a href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html#method.description\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.cause\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/error.rs.html#120\">source</a></span><a href=\"#method.cause\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html#method.cause\" class=\"fn\">cause</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;dyn <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt;</h4></section></summary><span class=\"item-info\"><div class=\"stab deprecated\"><span class=\"emoji\">👎</span><span>Deprecated since 1.33.0: replaced by Error::source, which can support downcasting</span></div></span></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.provide\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"https://doc.rust-lang.org/nightly/src/core/error.rs.html#184\">source</a><a href=\"#method.provide\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html#method.provide\" class=\"fn\">provide</a>&lt;'a&gt;(&amp;'a self, request: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/error/struct.Request.html\" title=\"struct core::error::Request\">Request</a>&lt;'a&gt;)</h4></section></summary><span class=\"item-info\"><div class=\"stab unstable\"><span class=\"emoji\">🔬</span><span>This is a nightly-only experimental API. (<code>error_generic_member_access</code>)</span></div></span><div class='docblock'>Provides type based access to context intended for error reports. <a href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html#method.provide\">Read more</a></div></details></div></details>","Error","alloy::transports::TransportError"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-From%3CE%3E-for-RpcError%3CE,+ErrResp%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#5\">source</a><a href=\"#impl-From%3CE%3E-for-RpcError%3CE,+ErrResp%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;E, ErrResp&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;E&gt; for <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.from\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#5\">source</a><a href=\"#method.from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html#tymethod.from\" class=\"fn\">from</a>(source: E) -&gt; <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;</h4></section></summary><div class='docblock'>Converts to this type from the input type.</div></details></div></details>","From<E>","alloy::transports::TransportError"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-RpcError%3CE,+ErrResp%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#56-58\">source</a><a href=\"#impl-RpcError%3CE,+ErrResp%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;E, ErrResp&gt; <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;<div class=\"where\">where\n    ErrResp: <a class=\"trait\" href=\"alloy/rpc/json_rpc/trait.RpcReturn.html\" title=\"trait alloy::rpc::json_rpc::RpcReturn\">RpcReturn</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.err_resp\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#61\">source</a><h4 class=\"code-header\">pub const fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.err_resp\" class=\"fn\">err_resp</a>(err: <a class=\"struct\" href=\"alloy/rpc/json_rpc/struct.ErrorPayload.html\" title=\"struct alloy::rpc::json_rpc::ErrorPayload\">ErrorPayload</a>&lt;ErrResp&gt;) -&gt; <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;</h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Instantiate a new <code>ErrorResp</code> from an error response.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.local_usage\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#66\">source</a><h4 class=\"code-header\">pub fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.local_usage\" class=\"fn\">local_usage</a>(\n    err: impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> + 'static\n) -&gt; <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;</h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Instantiate a new <code>LocalUsageError</code> from a custom error.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.local_usage_str\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#71\">source</a><h4 class=\"code-header\">pub fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.local_usage_str\" class=\"fn\">local_usage_str</a>(err: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>) -&gt; <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;</h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Instantiate a new <code>LocalUsageError</code> from a custom error message.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.deser_err\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#81\">source</a><h4 class=\"code-header\">pub fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.deser_err\" class=\"fn\">deser_err</a>(err: <a class=\"struct\" href=\"https://docs.rs/serde_json/1.0.117/serde_json/error/struct.Error.html\" title=\"struct serde_json::error::Error\">Error</a>, text: impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt;) -&gt; <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;</h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Instantiate a new <code>DeserError</code> from a <a href=\"https://docs.rs/serde_json/1.0.117/serde_json/error/struct.Error.html\" title=\"struct serde_json::error::Error\"><code>serde_json::Error</code></a> and the\ntext. This should be called when the error occurs during\ndeserialization.</p>\n<p>Note: This will check if the response is actually an <a href=\"alloy/rpc/json_rpc/struct.ErrorPayload.html\" title=\"struct alloy::rpc::json_rpc::ErrorPayload\">ErrorPayload</a>, if so it will return a\n<a href=\"alloy/transports/enum.RpcError.html#variant.ErrorResp\" title=\"variant alloy::transports::RpcError::ErrorResp\">RpcError::ErrorResp</a>.</p>\n</div></details></div></details>",0,"alloy::transports::TransportError"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-RpcError%3CE,+ErrResp%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#93\">source</a><a href=\"#impl-RpcError%3CE,+ErrResp%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;E, ErrResp&gt; <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.ser_err\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#96\">source</a><h4 class=\"code-header\">pub const fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.ser_err\" class=\"fn\">ser_err</a>(err: <a class=\"struct\" href=\"https://docs.rs/serde_json/1.0.117/serde_json/error/struct.Error.html\" title=\"struct serde_json::error::Error\">Error</a>) -&gt; <a class=\"enum\" href=\"alloy/transports/enum.RpcError.html\" title=\"enum alloy::transports::RpcError\">RpcError</a>&lt;E, ErrResp&gt;</h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Instantiate a new <code>SerError</code> from a <a href=\"https://docs.rs/serde_json/1.0.117/serde_json/error/struct.Error.html\" title=\"struct serde_json::error::Error\"><code>serde_json::Error</code></a>. This\nshould be called when the error occurs during serialization.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_ser_error\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#101\">source</a><h4 class=\"code-header\">pub const fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.is_ser_error\" class=\"fn\">is_ser_error</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Check if the error is a serialization error.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_deser_error\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#106\">source</a><h4 class=\"code-header\">pub const fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.is_deser_error\" class=\"fn\">is_deser_error</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Check if the error is a deserialization error.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_transport_error\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#111\">source</a><h4 class=\"code-header\">pub const fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.is_transport_error\" class=\"fn\">is_transport_error</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Check if the error is a transport error.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_error_resp\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#116\">source</a><h4 class=\"code-header\">pub const fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.is_error_resp\" class=\"fn\">is_error_resp</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Check if the error is an error response.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_null_resp\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#121\">source</a><h4 class=\"code-header\">pub const fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.is_null_resp\" class=\"fn\">is_null_resp</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Check if the error is a null response.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_unsupported_feature\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#126\">source</a><h4 class=\"code-header\">pub const fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.is_unsupported_feature\" class=\"fn\">is_unsupported_feature</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Check if the error is an unsupported feature error.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_local_usage_error\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#131\">source</a><h4 class=\"code-header\">pub const fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.is_local_usage_error\" class=\"fn\">is_local_usage_error</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Check if the error is a local usage error.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_error_resp\" class=\"method\"><a class=\"src rightside\" href=\"src/alloy_json_rpc/error.rs.html#136\">source</a><h4 class=\"code-header\">pub const fn <a href=\"alloy/transports/enum.RpcError.html#tymethod.as_error_resp\" class=\"fn\">as_error_resp</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;<a class=\"struct\" href=\"alloy/rpc/json_rpc/struct.ErrorPayload.html\" title=\"struct alloy::rpc::json_rpc::ErrorPayload\">ErrorPayload</a>&lt;ErrResp&gt;&gt;</h4></section><span class=\"item-info\"><div class=\"stab portability\">Available on <strong>crate features <code>rpc</code> and <code>json-rpc</code></strong> only.</div></span></summary><div class=\"docblock\"><p>Fallible conversion to an error response.</p>\n</div></details></div></details>",0,"alloy::transports::TransportError"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()