# parallel-gnuplot
Parallel calls to <a href="http://www.gnuplot.info/">GNUPlot</a>.
Calls the same <tt>GNUPlot</tt> script once for each data file block.
Please, note <tt>GNUPlot</tt> has copyrights,
and <tt>parallel-gnuplot</tt> is <strong>not</strong> a modified version of <tt>GNUPlot</tt>.

# GNUPlot variables
<tt>parallel-gnuplot</tt> sets some <tt>GNUPlot</tt> variables:
<ul>
<li><tt>INDEX</tt>: block index, starting at <tt>0</tt>;</li>
<li><tt>DATAFILE</tt>: path of a data file containing only a single block.</li>
</ul>

# Usage
<tt>parallel-gnuplot datafilename gpfilename [tmpdirectory]</tt>

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

You can call: <tt>parallel-gnuplot ./data.txt ./script.gp .</tt>

