## Route Example

```
$ evefrontier-cli route --from "Y:170N" --to "Z:46SO"

Route from Y:170N to Z:46SO (3 jumps):
 - Y:170N
 - B:40V6 (75ly via 4 gates)
 - Z:46SO (60ly via jump drive)

Total distance: 135ly
Total ly jumped: 60ly
```

## Route with Notepad Example

```
$ evefrontier-cli route --from "Y:170N" --to "Z:46SO" --format note
St <a href="showinfo:5//30000635">Y:170N</a>
SD <a href="showinfo:5//30000639">B:4OV6</a>
En <a href="showinfo:5//30007664">Z:46S0</a>
