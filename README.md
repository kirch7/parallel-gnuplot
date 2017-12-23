# parallel-gnuplot
Parallel calls to <a href="http://www.gnuplot.info/">GNUPlot</a>.
Calls the same `GNUPlot` script once for each data file block.
Please, note `GNUPlot` has copyrights,
and `parallel-gnuplot` is <strong>not</strong> a modified version of `GNUPlot`.

# GNUPlot variables
`parallel-gnuplot` sets some `GNUPlot` variables:
<ul>
<li>`INDEX`: block index, starting at `0`;</li>
<li>`DATAFILE`: path of a data file containing only a single block.</li>
</ul>

# Usage
`parallel-gnuplot datafilename gpfilename [tmpdirectory]`

# Example

data.txt:

```plain
# block 0:
0 0
1 1
2 2
3 3
4 4


# block 1:
0 0
1 2
2 4
3 6
4 8
```

script.gp:

```gnuplot
set terminal png size 800,600
set encoding utf8

set xrange [0:4]
set yrange [0:8]

set key left top
set output sprintf("%d", INDEX).'.png'

plot DATAFILE with lp lw 2 pt 7 ps 3 title sprintf("Block %d", INDEX)
```

You can call: `parallel-gnuplot ./data.txt ./script.gp ./`

