Kotoist
=======

VST plugin for live coding using [Koto](https://github.com/koto-lang/koto)
programming language.




## Build

Use `./build.sh win` on Windows or `./build.sh mac` on macOS.

If you wish to build manually, you should build GUI first:

```
cd gui
yarn
yarn build
```

Then you can build the plugin:

```
cd ..
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




## FL Studio Specific

The debug build initialize a log file on your desktop. At least on macOS this
crashes FL Studio on start. To prevent it, comment out the log initialization.




## Deployment

Just tag a new version and push it to remote.


### Docs

You need [doctave](https://github.com/Doctave/doctave) and **gh-pages** node
package.

To install **gh-pages** (it's important to use 3.0.0 version):

```
npm install -g gh-pages@3.0.0
```

Building and deploying the docs:

```
doctave build --release
gh-pages -d site
```
