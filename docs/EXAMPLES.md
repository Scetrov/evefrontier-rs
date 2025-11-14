## Route Example

```
$ evefrontier-cli route --from "Y:170N" --to "Z:46S0" --algorithm a-star

Route from Y:170N to Z:46S0 (3 jumps; algorithm: a-star):
 - Y:170N
 - M:4R8T (66ly via gate)
 - Y:1NV0 (23ly via jump)
 - Z:46S0 (48ly via jump)

Total distance: 137ly
Total ly jumped: 71ly
```

**Note:** Spatial jumps (jump drive) are only available with `--algorithm dijkstra` or `--algorithm a-star`. 
The default `bfs` algorithm only uses stargate connections.

## Route with Notepad Example

```
$ evefrontier-cli route --from "Y:170N" --to "Z:46S0" --algorithm a-star --format note
Sta <a href="showinfo:5//30000635">Y:170N</a>
Dst <a href="showinfo:5//30000639">M:4R8T</a>
Jmp <a href="showinfo:5//30007664">Z:46S0</a>
```

## Route with Emoji


```
$ evefrontier-cli route --from "Y:170N" --to "Z:46S0" --algorithm a-star --format emoji

Route from Y:170N to Z:46S0 (3 jumps):
 🚥 Y:170N
 📍 M:4R8T (66ly via gate)
 📍 Y:1NV0 (23ly via jump)
 🚀️ Z:46S0 (48ly via jump)

Total distance: 137ly
Total ly jumped: 71ly
```