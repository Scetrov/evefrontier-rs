## Route Example

```
$ evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --algorithm a-star

Route from ER1-MM7 to ENQ-PB6 (2 jumps; algorithm: a-star):
 - ER1-MM7 [min 6.57K]
 - IFM-228 [min 2.45K] (181ly via gate)
 - ENQ-PB6 [min 22.11K] (205ly via gate)

Total distance: 386ly
Total ly jumped: 0ly
```

**Note:** Spatial jumps (jump drive) are only available with `--algorithm dijkstra` or
`--algorithm a-star`. The default `bfs` algorithm only uses stargate connections.

## Route with Notepad Example

```
$ evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --algorithm a-star --format note
Sta <a href="showinfo:5//30001171">ER1-MM7</a>
Dst <a href="showinfo:5//30001177">IFM-228</a>
Jmp <a href="showinfo:5//30001176">ENQ-PB6</a>
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
$ evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --algorithm a-star --format emoji

Route from ER1-MM7 to ENQ-PB6 (2 jumps):
 🚥 ER1-MM7 [min 6.57K]
 📍 IFM-228 [min 2.45K] (181ly via gate)
 🚀️ ENQ-PB6 [min 22.11K] (205ly via gate)

Total distance: 386ly
Total ly jumped: 0ly
```

## Scout Examples

### Scout within range in lightyears from a starting point

This generates an optimized route to scout all systems within the specified range using the minimum
amount of fuel. Ensure that duplicate systems are clearly marked.

```
$ evefrontier-cli scout --from "ER1-MM7" --range 50 --format note
Scout from ER1-MM7 within 50ly:
 <a href="showinfo:5//30001177">IFM-228</a> (20ly via gate)
 <a href="showinfo:5//30001179">E85-NR6</a> (30ly via jump)
 <a href="showinfo:5//30001180">IR5-K72</a> (45ly via gate)
```

### Scout showing only stargate connections

Same as above but only using stargate connections,again should optimize for minimum fuel usage /
jumps to cover all stars and return to the start.

```$ evefrontier-cli scout --from "ER1-MM7" --gates-only
Scout from ER1-MM7 within 50ly (gates only):
 - IFM-228 (20ly via gate)
 - IR5-K72 (45ly via gate)
```

> [!NOTE] Other formats such as `emoji` and `table` are also supported for the `scout` command.
