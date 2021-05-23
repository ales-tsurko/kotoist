Kotoist
=======

VST plugin for live coding using [Koto](https://github.com/koto-lang/koto)
programming language.




## Build

[ref](https://github.com/RustAudio/vst-rs#packaging-on-os-x)

On OS X VST plugins are packaged inside of loadable bundles. To package your VST
as a loadable bundle you may use the osx_vst_bundler.sh script this library
provides. 

Example: 

```
./osx_vst_bundler.sh Plugin target/release/plugin.dylib
Creates a Plugin.vst bundle
```
