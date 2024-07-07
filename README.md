# but

一个文件夹实时备份工具，使用 *Rust* 编写。

该程序是 [WSMCS 服务器](https://www.wsmcs.top) 工具的一部分。

## 使用

打开 Actions 页面，下载最新发布的二进制文件。

## 软件配置

### 配置文件示例
```json
{
  "backup": [
    {
      "name": "save",
      "from": "/home/mcseekeri/.local/share/PrismLauncher/instances/WSMCS/.minecraft/saves/",
      "dest": "./"
    },
    {
      "name": "Server",
      "from": "/opt/MCSManager/data/InstanceData/",
      "dest": "./"
    }
  ],
  "settings": {
    "interval": 300,
    "saving_name": "%name%-%timestamp%",
    "compression": "zstd"
  }
}
```
## 配置文件位置
but 将依次在 `/etc/but.json` `$HOME/.config/but.json` 和 `./but.json` 三个位置寻找配置文件，优先级从高到低。

### 作为系统服务运行

要作为系统服务运行，你需要将 `but` 放在 `/usr/local/bin` 目录下。为了减少工作量，你可以执行 `ln -s <but的完整目录>  /usr/local/bin/but` 以创建软链接。

将 `but.service` 文件复制到 `/etc/systemd/system/` 目录，并执行以下命令：

```bash
systemctl daemon-reload
systemctl enable --now but
```

> 如果启动出错，可以输入`systemctl status but`查看错误日志。

## 贡献者（排名不分先后）

<a href="https://github.com/SO-TS/but/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=SO-TS/but" />
</a>
