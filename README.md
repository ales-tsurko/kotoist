Kotoist
=======

VST plugin for live coding using [Koto](https://github.com/koto-lang/koto)
programming language.




## Build

Use `./build.sh win` on Windows or `./build.sh mac` on macOS.

If you wish to build manually, you should build GUI first:

```
cd gui
yarn build
```

Then you can build the plugin:

```
cargo build
```

On OS X VST plugins are packaged inside of loadable bundles. To package your VST
as a loadable bundle you may use the osx_vst_bundler.sh script this library
provides. 

Example: 

```
./osx_vst_bundler.sh Plugin target/release/plugin.dylib
Creates a Plugin.vst bundle
```

[ref](https://github.com/RustAudio/vst-rs#packaging-on-os-x)




## REAPER Specific

To make the plugin work as expected, you should right-click on the plugin in the
FX Rack and choose "Send all keyboard input to plugin".
