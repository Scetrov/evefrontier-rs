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

**Note:** Spatial jumps (jump drive) are only available with `--algorithm dijkstra` or
`--algorithm a-star`. The default `bfs` algorithm only uses stargate connections.

## Route with Notepad Example

```
$ evefrontier-cli route --from "Y:170N" --to "Z:46S0" --algorithm a-star --format note
Sta <a href="showinfo:5//30000635">Y:170N</a>
Dst <a href="showinfo:5//30000639">M:4R8T</a>
Jmp <a href="showinfo:5//30007664">Z:46S0</a>
```

## Route with emoji++ view


```
$ evefrontier-cli route --from EKB-F37 --to ECS-9R2 --format enhanced
Route from EKB-F37 to ECS-9R2 (6 jumps):
 🚥 EKB-F37
 | [min 0.97K, 3 Planets, 6 Moons]
 🚀 IR7-L76 (75ly jump)
 | [min 4.03K, 4 Planets, 15 Moons]

Total distance: 460ly
Total ly jumped: 460ly

Completed in 6.53s
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

## Scout Examples

### Scout within range in lightyears from a starting point

This generates an optimized route to scout all systems within the specified range using the minimum
amount of fuel. Ensure that duplicate systems are clearly marked.

```
$ evefrontier-cli scout --from "Strym" --range 50 --format note
Scout from Strym within 50ly:
 <a href="showinfo:5//30000143">Onga</a> (20ly via gate)
 <a href="showinfo:5//30000144">Niarja</a> (30ly via jump)
 <a href="showinfo:5//30000145">Halaima</a> (45ly via gate)
```

### Scout showing only stargate connections

Same as above but only using stargate connections,again should optimize for minimum fuel usage /
jumps to cover all stars and return to the start.

```$ evefrontier-cli scout --from "Strym" --gates-only
Scout from Strym within 50ly (gates only):
 - Onga (20ly via gate)
 - Halaima (45ly via gate)
```

> [!NOTE] Other formats such as `emoji` and `table` are also supported for the `scout` command.
