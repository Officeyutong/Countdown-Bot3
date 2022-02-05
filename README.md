# Countdown-Bot-3

使用Rust重写的咕咕倒计时，编译后单文件配合go-cqhttp即可执行，不需要其他外部依赖。

## 一些东西
- Github Actions里有编译好的程序，可以自己下载
- 插件`zxhdmx`的核心逻辑为Python编写。此插件通过RustPython（一个Rust写的Python解释器）运行Python代码。
- 要使用插件music_163，需要https://github.com/Binaryify/NeteaseCloudMusicApi
- 要使用插件docker_runner和math，需要docker并且构建好docker文件夹下的镜像
- 要使用插件ds_drawer_plugin，需要安装graphviz
- 要使用插件read，需要百度语音合成的API
- 要使用插件weather，需要和风天气的API_KEY

