[English](https://github.com/ReyReyy/tcpping/blob/master/README.md) | 中文

# Tcp ping

像 “ping” 命令一样测试服务器延迟，但使用 tcp 协议！

## 用法

```
tcpping [选项] <目标 IP/Host>
```

Ctrl + C 停止。

## 选项

```
-p <port>               指定端口
-4                      使用 IPv4
-6                      使用 IPv6
-c <count>              执行 <count> 次后停止
-v                      显示版本
-h                      显示帮助
```

默认:

- 端口: 80
- IP 版本: 系统默认 IP 优先级
- 次数: 0（无限）

## 如何安装

- ### Linux & MacOS

  你可以通过运行一个简单的命令来安装它：

  ```
  sudo bash -c "$(curl -L -s tcpping.reyreyy.net)"
  ```

- ### Windows

  从 [发布页面](https://github.com/ReyReyy/tcpping/releases) 下载正式版

  解压 zip 文件然后运行 `tcpping.exe`

## 卸载

- ### Linux & MacOS

  使用 `rm` 命令卸载：

  ```
  sudo rm -f /usr/local/bin/tcpping
  ```

- ### Windows

  删除 `tcpping.exe` 文件就行

## 示例：

使用默认配置：

```
reyreyy@ReMac ~ % tcpping google.com
TCP PING google.com [2404:6800:4012:5::200e]:80
Connected to [2404:6800:4012:5::200e]:80, tcp_seq=0 time=5.673 ms
Connected to [2404:6800:4012:5::200e]:80, tcp_seq=1 time=3.692 ms
Connected to [2404:6800:4012:5::200e]:80, tcp_seq=2 time=3.827 ms
Connected to [2404:6800:4012:5::200e]:80, tcp_seq=3 time=3.391 ms
^C
--- google.com tcp ping statistics ---
4 packets transmitted, 4 packets received, 0.0% packet loss
round-trip min/avg/max/stddev = 3.391/4.146/5.673/0.896 ms
```

指定端口和 IP 版本：

```
reyreyy@ReMac ~ % tcpping google.com -p 443 -4
TCP PING google.com 142.250.77.14:443
Connected to 142.250.77.14:443, tcp_seq=0 time=4.080 ms
Connected to 142.250.77.14:443, tcp_seq=1 time=3.981 ms
Connected to 142.250.77.14:443, tcp_seq=2 time=3.272 ms
Connected to 142.250.77.14:443, tcp_seq=3 time=3.367 ms
^C
--- google.com tcp ping statistics ---
4 packets transmitted, 4 packets received, 0.0% packet loss
round-trip min/avg/max/stddev = 3.272/3.675/4.080/0.359 ms
```

## 感谢

感谢 [cursor IDE](https://www.cursor.com/)<br>
这个项目 90% 是由 cursor IDE 生成的<br>
其实我完全不会写 rust 代码 :P<br>

~~呜呜呜！cursor IDE 试用时间结束了！没 AI 用了！(;´༎ຶД༎ຶ`)~~
