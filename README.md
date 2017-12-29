# parallel-gnuplot
[![Version info](https://img.shields.io/crates/v/parallel-gnuplot.svg)](https://crates.io/crates/parallel-gnuplot)
[![Build Status](https://travis-ci.org/kirch7/parallel-gnuplot.svg?branch=master)](https://travis-ci.org/kirch7/parallel-gnuplot)
[![Build status](https://ci.appveyor.com/api/projects/status/9ipxmpbme70mgi2j/branch/master?svg=true)](https://ci.appveyor.com/project/kirch7/parallel-gnuplot/branch/master)

Parallel calls to <a href="http://www.gnuplot.info/">GNUPlot</a>.
Calls the same `GNUPlot` script once for each data file block.
Please, note `GNUPlot` has copyrights,
and `parallel-gnuplot` is <strong>not</strong> a modified version of `GNUPlot`.

### GNUPlot variables
`parallel-gnuplot` sets some `GNUPlot` variables:
<ul>
<li> <tt>INDEX</tt>: block index, starting at <tt>0</tt>; </li>
<li> <tt>DATAFILE</tt>: path of a data file containing only a single block. </li>
</ul>

### Usage
`parallel-gnuplot datafilename gpfilename [tmpdirectory]`

or

`program_outputing_data | parallel-gnuplot gpfilename [tmpdirectory]`

or

`cargo run --release -- datafilename gpfilename [tmpdirectory]`

where `[tmpdirectory]` is optional.

### Example

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

You can call:
  `parallel-gnuplot ./data.txt ./script.gp`
  or
  `cargo run --release -- ./data.txt ./script.gp`
  or something like
  `cat ./data.txt | parallel-gnuplot ./script.gp`

### Features
<ul>
  <li>Tested with the Operating Systems:
  <ul>
    <li><em>MS Windows</em> (works since v0.1.4),</li>
    <li><em>GNU/Linux</em>.</li>
    <li>(Let me know if works in other OSs.)</li>
  </ul>
  <li>Can receive data through pipe (since v0.1.5).</li>
  <li>Check vality of script (since v0.1.6).</li>
</Ul>
