inputs:
[
  (import inputs.rust-overlay)
  (final: prev: {
    bundler = prev.bundler.override { ruby = final.ruby_3_3; };
    bundix = prev.bundix.overrideAttrs (oldAtts: {
      ruby = final.ruby_3_3;
    });
    ruby_3_3 = ((final.mkRuby {
      version = final.mkRubyVersion "3" "3" "6" "";
      hash = "sha256-jcSP/68nD4bxAZBT8o5R5NpMzjKjZ2CgYDqa7mfX/Y0=";
      cargoHash = "sha256-GeelTMRFIyvz1QS2L+Q3KAnyQy7jc0ejhx3TdEFVEbk=";
    }).override
      {
        jemallocSupport = true;
      });
    craneLib = (inputs.crane.mkLib final).overrideToolchain final.rust-bin.stable.latest.default;
    rust-toolchain = prev.rust-bin.stable.latest.default;
    # This is an extended rust toolchain with `rust-src` since that's required for IDE stuff
    rust-dev-toolchain = prev.rust-bin.stable.latest.default.override {
      extensions = [ "rust-src" ];
    };
  })
]
