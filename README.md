```
hey -n 1000000 -c 250 http://gateway:8080

Summary:
  Total:        4.1712 secs
  Slowest:      0.0840 secs
  Fastest:      0.0001 secs
  Average:      0.0010 secs
  Requests/sec: 239736.3098


Response time histogram:
  0.000 [1]     |
  0.008 [998645]        |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.017 [1287]  |
  0.025 [64]    |
  0.034 [0]     |
  0.042 [0]     |
  0.050 [0]     |
  0.059 [0]     |
  0.067 [0]     |
  0.076 [0]     |
  0.084 [3]     |


Latency distribution:
  10% in 0.0004 secs
  25% in 0.0005 secs
  50% in 0.0008 secs
  75% in 0.0012 secs
  90% in 0.0018 secs
  95% in 0.0026 secs
  99% in 0.0048 secs

Details (average, fastest, slowest):
  DNS+dialup:   0.0000 secs, 0.0001 secs, 0.0840 secs
  DNS-lookup:   0.0000 secs, 0.0000 secs, 0.0799 secs
  req write:    0.0000 secs, 0.0000 secs, 0.0042 secs
  resp wait:    0.0008 secs, 0.0000 secs, 0.0785 secs
  resp read:    0.0001 secs, 0.0000 secs, 0.0816 secs

Status code distribution:
  [204] 1000000 responses
```


```
hey -n 1000000 -c 250 http://nginx:80

Summary:
  Total:        6.6304 secs
  Slowest:      0.0827 secs
  Fastest:      0.0000 secs
  Average:      0.0016 secs
  Requests/sec: 150821.4629

  Total data:   1000000 bytes
  Size/request: 1 bytes

Response time histogram:
  0.000 [1]     |
  0.008 [985842]        |■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■■
  0.017 [9956]  |
  0.025 [2654]  |
  0.033 [534]   |
  0.041 [144]   |
  0.050 [331]   |
  0.058 [343]   |
  0.066 [147]   |
  0.074 [34]    |
  0.083 [14]    |


Latency distribution:
  10% in 0.0004 secs
  25% in 0.0007 secs
  50% in 0.0011 secs
  75% in 0.0017 secs
  90% in 0.0030 secs
  95% in 0.0044 secs
  99% in 0.0105 secs

Details (average, fastest, slowest):
  DNS+dialup:   0.0000 secs, 0.0000 secs, 0.0827 secs
  DNS-lookup:   0.0000 secs, 0.0000 secs, 0.0453 secs
  req write:    0.0000 secs, 0.0000 secs, 0.0434 secs
  resp wait:    0.0015 secs, 0.0000 secs, 0.0826 secs
  resp read:    0.0001 secs, 0.0000 secs, 0.0478 secs

Status code distribution:
  [200] 1000000 responses
```