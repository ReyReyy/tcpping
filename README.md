# Tcp ping

Testing server latency like command "ping", but with tcp!

## Usage

```
tcpping <IP Address> [SUBCOMMAND]
```

Ctrl + C to stop.

## Subcommands

```
[-p | --port] <port>               Add port
[-4 | --IPv4]                      Force IPv4
[-6 | --IPv6]                      Force IPv6
[-c | --count] <count>             Test <count> times, 0 is unlimited.
[-v | --versioin]                  Show version
[-h | --help]                      Show help
```

Default:

- Port: 80
- IP Version: IPv6
- Count: 0 (Unlimited)

## How to Install (Recommend)

Put the `tcpping` file in to `/usr/local/bin` <br>
Then you can use it in your Terminal as a normal command :)

## Sample:

Use as default:

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

Force port and IP version:

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
