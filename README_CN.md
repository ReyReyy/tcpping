[English](https://github.com/ReyReyy/tcpping/blob/master/README.md) | 中文

# Tcp ping

像 “ping” 命令一样测试服务器延迟，但使用 tcp 协议！

## 用法

```
tcpping <IP 地址> [子命令]
```

Ctrl + C 停止。

## 子命令

```
[-p | --port] <port>               添加端口
[-4 | --IPv4]                      强制使用 IPv4
[-6 | --IPv6]                      强制使用 IPv6
[-c | --count] <count>             测试 <count> 次，0 是不限制次数。
[-v | --version]                   显示版本
[-h | --help]                      显示帮助
```

默认:

- 端口: 80
- IP 版本: IPv6
- 次数: 0（无限）

## 如何安装（推荐）

将 `tcpping` 文件放入 `/usr/local/bin` <br>
然后你就可以在终端中像正常命令一样使用它 :)

## 示例：

使用默认配置：

```
reyreyy@ReMac ~ % tcpping google.com
TCP PING google.com (2404:6800:4012::200e):80
Connected to 2404:6800:4012::200e:80: tcp_seq=0 time=2.676 ms
Connected to 2404:6800:4012::200e:80: tcp_seq=1 time=3.640 ms
Connected to 2404:6800:4012::200e:80: tcp_seq=2 time=4.629 ms
Connected to 2404:6800:4012::200e:80: tcp_seq=3 time=6.308 ms
^C
--- google.com tcp ping statistics ---
4 packets transmitted, 4 packets received, 0.0% packet loss
round-trip min/avg/max/stddev = 2.676/4.313/6.308/1.343 ms
```

指定端口和 IP 版本：

```
reyreyy@ReMac ~ % tcpping google.com -p 443 -4
TCP PING google.com (172.217.160.78):443
Connected to 172.217.160.78:443: tcp_seq=0 time=3.816 ms
Connected to 172.217.160.78:443: tcp_seq=1 time=3.828 ms
Connected to 172.217.160.78:443: tcp_seq=2 time=3.297 ms
Connected to 172.217.160.78:443: tcp_seq=3 time=5.488 ms
^C
--- google.com tcp ping statistics ---
4 packets transmitted, 4 packets received, 0.0% packet loss
round-trip min/avg/max/stddev = 3.297/4.107/5.488/0.825 ms
```