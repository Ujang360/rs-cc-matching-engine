# CC Matching Engine

![Build Status](https://github.com/Ujang360/rs-cc-matching-engine/workflows/Build/badge.svg)

## Local Run (Laptop)

### System Setup

```text
            .-/+oossssoo+/-.               kresna@funky-laptop
        `:+ssssssssssssssssss+:`           -------------------
      -+ssssssssssssssssssyyssss+-         OS: Ubuntu 18.04.3 LTS x86_64
    .ossssssssssssssssssdMMMNysssso.       Host: 2447D18 ThinkPad W530
   /ssssssssssshdmmNNmmyNMMMMhssssss/      Kernel: 5.0.0-36-lowlatency
  +ssssssssshmydMMMMMMMNddddyssssssss+     Uptime: 1 hour, 5 mins
 /sssssssshNMMMyhhyyyyhmNMMMNhssssssss/    Packages: 2230
.ssssssssdMMMNhsssssssssshNMMMdssssssss.   Shell: bash 4.4.20
+sssshhhyNMMNyssssssssssssyNMMMysssssss+   Resolution: 1920x1080
ossyNMMMNyMMhsssssssssssssshmmmhssssssso   DE: Unity
ossyNMMMNyMMhsssssssssssssshmmmhssssssso   WM: GNOME Shell
+sssshhhyNMMNyssssssssssssyNMMMysssssss+   WM Theme: Adwaita
.ssssssssdMMMNhsssssssssshNMMMdssssssss.   Theme: Adwaita-dark [GTK2/3]
 /sssssssshNMMMyhhyyyyhdNMMMNhssssssss/    Icons: Ubuntu-mono-dark [GTK2/3]
  +sssssssssdmydMMMMMMMMddddyssssssss+     Terminal: vscode
   /ssssssssssshdmNNNNmyNMMMMhssssss/      CPU: Intel i7-3720QM (8) @ 3.600GHz
    .ossssssssssssssssssdMMMNysssso.       GPU: NVIDIA Quadro K2000M
      -+sssssssssssssssssyyyssss+-         GPU: Intel 3rd Gen Core processor Graphics Controlle
        `:+ssssssssssssssssss+:`           Memory: 6522MiB / 23916MiB
            .-/+oossssoo+/-.
```

### Result

```text

CC Matching Engine (Rust) - 1.0.0-beta.0
========================================

[Data Structure Alignment]
- OrderEvent:
  * Size 64 bytes
  * Aligment 8 bytes
- OrderMessage:
  * Size 120 bytes
  * Aligment 8 bytes
- OrderbookOrder:
  * Size 24 bytes
  * Aligment 8 bytes
- Orderbook:
  * Size 80 bytes
  * Aligment 8 bytes
- Orderbooks:
  * Size 216 bytes
  * Aligment 8 bytes

[Benchmark: 5 Limit Match (10 Orders)]
- Populating Orders...DONE
- Matching...DONE
- Took 12132 ns to complete
- 824402 Orders per second
- Orderbook Bids count 0 orders
- Orderbook Asks count 0 orders

[Benchmark: 50 Limit Match (100 Orders)]
- Populating Orders...DONE
- Matching...DONE
- Took 69862 ns to complete
- 1432664 Orders per second
- Orderbook Bids count 0 orders
- Orderbook Asks count 0 orders

[Benchmark: 500 Limit Match (1000 Orders)]
- Populating Orders...DONE
- Matching...DONE
- Took 700963 ns to complete
- 1428571 Orders per second
- Orderbook Bids count 0 orders
- Orderbook Asks count 0 orders

[Benchmark: 5000 Limit Match (10000 Orders)]
- Populating Orders...DONE
- Matching...DONE
- Took 6642319 ns to complete
- 1506024 Orders per second
- Orderbook Bids count 0 orders
- Orderbook Asks count 0 orders

[Benchmark: 50000 Limit Match (100000 Orders)]
- Populating Orders...DONE
- Matching...DONE
- Took 73429476 ns to complete
- 1362397 Orders per second
- Orderbook Bids count 0 orders
- Orderbook Asks count 0 orders

[Benchmark: 500000 Limit Match (1000000 Orders)]
- Populating Orders...DONE
- Matching...DONE
- Took 884735682 ns to complete
- 1131221 Orders per second
- Orderbook Bids count 0 orders
- Orderbook Asks count 0 orders

```

## Authors

- [Aditya Kresna](https://github.com/ujang360)
