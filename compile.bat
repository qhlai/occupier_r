cargo build --release
del/f/s/q  ./compiled_pack/occupier_r_win.exe
ren .\target\release\occupier_r.exe occupier_r_win.exe
move .\target\release\occupier_r_win.exe .\compiled_pack