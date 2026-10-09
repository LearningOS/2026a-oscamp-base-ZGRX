# mutex_counter 笔记

这个练习要做的事情是：

```text
多个线程共享同一个数据，并且安全地修改它。
```

核心组合是：

```rust
Arc<Mutex<T>>
```

可以先这样理解：

```text
Arc   ：让多个线程都能拿到同一份数据
Mutex ：给数据加一把锁，同一时间只允许一个线程修改
```

所以：

```rust
Arc<Mutex<usize>>
```

就是：

```text
多个线程共享的、被锁保护的 usize 计数器
```

## 为什么需要 Mutex

如果多个线程同时修改同一个数字，会出问题。

`+1` 看起来是一句代码，但实际大概分成三步：

```text
读当前值
加 1
写回去
```

如果两个线程同时操作，可能发生：

```text
当前值是 0

线程 A 读到 0
线程 B 读到 0
线程 A 写回 1
线程 B 写回 1
```

明明加了两次，最后却还是 `1`。

所以要用 `Mutex`：

```rust
let mut num = c.lock().unwrap();
*num += 1;
```

`lock()` 的意思是：

```text
我现在要修改这个数据，其他线程先等一下。
```

锁不是永久占用。`num` 离开作用域后，锁会自动释放。

例如：

```rust
for _ in 0..count_per_thread {
    let mut num = c.lock().unwrap();
    *num += 1;
} // 这一轮循环结束，num 消失，锁释放
```

## 为什么需要 Arc

`Mutex` 负责保护数据，但还不够。

多个线程都要使用同一个计数器，普通变量不能被同时 move 到多个线程里。

所以要用：

```rust
let counter = Arc::new(Mutex::new(0usize));
```

每开一个线程前，克隆一份 `Arc`：

```rust
let c = Arc::clone(&counter);
```

这里不是复制一个新的计数器，而是复制一个指向同一个计数器的共享指针。

也就是说：

```text
counter 和 c 指向同一个 Mutex<usize>
```

## 怎么开多个线程

开多个线程的套路是：

```rust
let mut handles = Vec::new();

for _ in 0..n_threads {
    let handle = thread::spawn(move || {
        // 子线程要做的事情
    });

    handles.push(handle);
}
```

`handles` 用来保存每个线程的句柄。

后面要通过这些句柄等待线程结束：

```rust
for handle in handles {
    handle.join().unwrap();
}
```

## concurrent_counter 的思路

函数：

```rust
pub fn concurrent_counter(n_threads: usize, count_per_thread: usize) -> usize
```

要做的是：

```text
开 n_threads 个线程
每个线程把同一个 counter 加 count_per_thread 次
最后返回 counter 的最终值
```

测试：

```rust
assert_eq!(concurrent_counter(10, 100), 1000);
```

意思是：

```text
10 个线程，每个线程加 100 次，最终应该是 1000
```

基本模板：

```rust
let counter = Arc::new(Mutex::new(0usize));
let mut handles = Vec::new();

for _ in 0..n_threads {
    let c = Arc::clone(&counter);

    let handle = thread::spawn(move || {
        for _ in 0..count_per_thread {
            let mut num = c.lock().unwrap();
            *num += 1;
        }
    });

    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

let result = *counter.lock().unwrap();
result
```

## concurrent_collect 的思路

函数：

```rust
pub fn concurrent_collect(n_threads: usize) -> Vec<usize>
```

要做的是：

```text
开 n_threads 个线程
每个线程把自己的 id push 到共享 Vec 里
最后排序并返回 Vec
```

这里用：

```rust
Arc<Mutex<Vec<usize>>>
```

因为多个线程要共同修改同一个 `Vec`。

为什么最后要排序？

因为线程执行顺序不确定。

可能先执行线程 3，再执行线程 1，所以原始结果可能是：

```rust
vec![3, 1, 0, 4, 2]
```

测试想要稳定结果：

```rust
vec![0, 1, 2, 3, 4]
```

所以最后要：

```rust
result.sort();
```

## 这题最重要的模板

```rust
let shared = Arc::new(Mutex::new(initial_value));
let mut handles = Vec::new();

for i in 0..n_threads {
    let shared_clone = Arc::clone(&shared);

    let handle = thread::spawn(move || {
        let mut data = shared_clone.lock().unwrap();
        // 修改 data
    });

    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}
```

一句话总结：

```text
Arc 负责共享
Mutex 负责保护
lock 负责进入共享数据
join 负责等线程结束
```
