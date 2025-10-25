# prism3-rust-atomic

易用的原子操作封装，提供类似 JDK atomic 包的 API。

## 设计目标

- **易用性**：隐藏内存序复杂性，提供合理的默认内存序
- **完整性**：提供与 JDK atomic 类似的高级操作方法
- **安全性**：保证内存安全和线程安全
- **性能**：零成本抽象，不引入额外开销
- **灵活性**：通过 `inner()` 方法暴露底层类型，高级用户可直接操作标准库类型

## 计划支持的类型

- `AtomicBool` - 原子布尔值
- `AtomicI32` - 32位有符号整数
- `AtomicI64` - 64位有符号整数
- `AtomicU32` - 32位无符号整数
- `AtomicU64` - 64位无符号整数
- `AtomicIsize` - 指针大小的有符号整数
- `AtomicUsize` - 指针大小的无符号整数
- `AtomicRef<T>` - 原子引用

## 文档

详细设计文档请参见：[doc/atomic_design_zh_CN_v1.0.claude.md](doc/atomic_design_zh_CN_v1.0.claude.md)

## 作者

胡海星

## 许可证

MIT OR Apache-2.0

