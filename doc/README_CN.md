# 资源占用
尽量保持所有剩余资源在一个设定的比例。

## 注意！ 中文文档更新慢一点，问就是懒

# 特点:

按设定比例占用计算机资源，动态调整软件占用，正常软件占用时自动释放，尽量不影响正常使用。

可用于测试虚拟服务器提供商提供的真实资源，这可能毫无意义。

## 使用方法
例子:

占用60%资源

对于 linux_x86_64
```bash
wget https://github.com/qhlai/occupier_r/raw/master/compiled_pack/occupier_r_linux_x86_64
chmod +x ./occupier_r_linux_x86_64
./occupier_r_linux_x86_64  -c 60 -m 60 -s 60
```
对于  win10 64位
```bash
wget https://github.com/qhlai/occupier_r/raw/master/compiled_pack/occupier_r_win.exe
./occupier_r_win.exe  -c 60 -m 60 -s 60
```

## 参数
-c 处理器 (待完成)

-s 硬盘

-m 内存

更多参数可看 [src/occupier_r.yaml](https://github.com/qhlai/occupier_r/blob/master/src/occupier_r.yaml)

## 待完成

support CPU (不安全，可能永远不会做)

存储器随机读写(会影响性能，与不影响正常使用初衷不符)

## 注意

小心使用，不承担任何责任。

