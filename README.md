# Resourse Occupier 
Is to try to keep all remaining resources in a set proportion.

## Feature:

Occupy computer resources according to the set proportion, dynamically adjust the software occupation, and automatically release when the normal software occupation, so as not to affect the normal use as much as possible.


It can be used to test the real resources provided by the virtual server provider, which may be meaningless.


## USAGE
example:

60% of resources

for linux_x86_64
```bash
wget https://github.com/qhlai/occupier_r/releases/download/0.1.1/occupier_r_x86_64
chmod +x ./occupier_r_linux_x86_64
./occupier_r_linux_x86_64 -m 60 -s 60
```

for linux_aarch64
```bash
wget https://github.com/qhlai/occupier_r/releases/download/0.1.1/occupier_r_aarch64
chmod +x ./occupier_r_linux_aarch64
./occupier_r_linux_x86_64 -m 60 -s 60
```

may not support win10

resourse occupier
## Parameter
-c cpu (TODO)

-s storage

-m memory

## TODO

not support CPU (Not very safe, I may not support it soon or never)

random read/write occupy for memory/storage 

## NOTE

Use with caution and bear the consequences.

The author assumes no responsibility for any loss.

Do not use for important equipment.