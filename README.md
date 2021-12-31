# Resourse Occupier 
Is to try to keep all remaining resources in a set proportion.
## Feature:

Occupy computer resources according to the set proportion, dynamically adjust the software occupation, and automatically release when the normal software occupation, so as not to affect the normal use as much as possible.


It can be used to test the real resources provided by the virtual server provider, which may be meaningless.


## USAGE
example:

60% of resources

for linux
```bash
wget https://github.com/qhlai/occupier_r/master/compiled_pack/occupier_r_linux
chmod +x ./occupier_r_linux
./occupier_r_linux  -c 60 -m 60 -s 60
```
for windows
```bash
wget https://github.com/qhlai/occupier_r/master/compiled_pack/occupier_r_win.exe
./occupier_r_win.exe  -c 60 -m 60 -s 60
```
resourse occupier
## Parameter
-c cpu (TODO)

-s storage

-m memory

More Parameter can see in occupier_r.yaml in this folder

## TODO

support CPU (Not very safe, I may not support it soon or never)

random read/write occupy for memory/storage 

## NOTE

Use with caution and bear the consequences.

The author assumes no responsibility for any loss.

Do not use for important equipment.
