# bump_allocator 笔记

这题要在一段连续内存 `heap_start..heap_end` 中实现最简单的分配器。`next` 指向尚未分配区域的起点。每次分配只向前移动 `next`，因此叫 Bump Allocator。

```text
heap_start       next                         heap_end
| 已分配的内存   | 可用内存                    |
                 ↑ 对齐后从这里分配
```

## 做题思路

1. 从原子变量 `next` 读出当前地址 `current`。
2. 按 `layout.align()` 向上对齐，得到 `aligned`。对齐量一定是 2 的幂，所以可以用 `(current + align - 1) & !(align - 1)` 清掉低位。
3. 计算 `end = aligned + layout.size()`。如果地址加法溢出，或 `end > heap_end`，返回空指针。用 `checked_add` 防止溢出后错误地通过边界检查。
4. 用 `compare_exchange(current, end, ...)` 抢占这段内存。成功则返回 `aligned`；失败说明别的线程已经移动了 `next`，需要从新地址重新对齐并检查边界。

例如 `current = 0x1003`、`align = 8`、`size = 16`：对齐后起点为 `0x1008`，终点为 `0x1018`。中间的 5 字节填充无法由这个分配器回收。

## 考察点

- **`GlobalAlloc` 和 `Layout`**：`alloc` 根据大小和对齐要求返回指针；容量不足时返回空指针。
- **地址对齐**：返回地址必须满足 `aligned % layout.align() == 0`，并且对齐填充也会占用堆空间。
- **整数溢出与边界**：先检查加法，再比较 `heap_end`；不能让地址回绕后被误认为仍在堆内。
- **原子 CAS 循环**：普通的 `load` 加 `store` 会让两个线程同时拿到同一段空间。CAS 保证只有一个线程能把观察到的旧 `next` 改成 `end`。这里沿用题目要求的 `SeqCst` 顺序。
- **释放策略**：`dealloc` 不回收单个对象；`reset` 把整个分配器恢复到起点。调用 `reset` 前必须确认旧分配都不再使用，且没有线程正在分配，否则旧地址可能被重复分配。

运行本题测试：`cargo test -p bump_allocator`。
