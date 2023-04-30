<p align="center"><img width="70%" height="70%" src="additional/icon/banner.png" alt="rycon"></p>

An open source programming language for web development with expressive type system and easy-to-learn syntax that makes it easy to build reliable and efficient software.

Example of hello world program:

<pre>
<b>import</b> std.io.println;

<b>pub</b> <b>fun</b> main() {
    println("hello world");
}
</pre>

No nulls, we use option types!

<pre>
<b>pub</b> <b>fun</b> div[T](a: T, b: T): <b>Option</b>[T] <b>where</b> T: <b>Numeric</b> {
    <b>if</b> b <b>==</b> <b>0</b> {
        <b>None</b>
    } <b>else</b> {
        <b>Some</b>(a / b)
    }
}
</pre>

We use result types as well with `unwrap_or` and postfix `?` and operator!

<pre>
<b>import</b> std.fs.File;

<b>pub</b> <b>fun</b> main() {
    <b>var</b> a = File.open(<i>"test.txt"</i>)<b>?</b>; // returns (Unit type in this case) if error will occur
    <b>var</b> num = <i>"27"</i>.parse[<b>i32</b>]().unwrap_or(<b>0</b>); // if error will occur, num will be set to 0
}
</pre>

We use traits like in Rust!

<pre>
// example of auto trait
<b>impl</b>[T] Test <b>for</b> T {}
<b>impl</b>[T] <b>Negative</b>[Test] <b>for</b> <b>Option</b>[T] {} // trait will NOT be implemented for options
<b>imp</b>l[T] <b>Negative</b>[Test] <b>for</b> T <b>where</b> T: <b>Default</b> {} // trait will NOT be implemented for types implementing Default
</pre>

Sum types, dynamic trait dispatchers as well as type aliases!

<pre>
<b>pub</b> <b>type</b> A = <b>Sum</b>[B, C];
<b>pub</b> <b>type</b> E = <b>Satisfies</b>[D, F];
</pre>

# Builds

<table style="margin-left: auto; margin-right: auto;">
<tr>
<td>Linux - Ubuntu (latest)</td>
<td>

![](https://img.shields.io/github/actions/workflow/status/abs0luty/ry/ry-ubuntu.yml)

</td>
</tr>
<tr>
<td>Windows (latest)</td>
<td>

![](https://img.shields.io/github/actions/workflow/status/abs0luty/ry/ry-windows.yml)

</td>
</tr>
</table>

# Installation

## Compiling from source code

You need to have Rust installed on your system. Then run:

<pre>
<b>cargo</b> install --path crates/ry
</pre>

Then you're good to go coding in Ry!

# Documentation

> Not made

# Architecture

[![](https://mermaid.ink/img/pako:eNptUk1v4jAQ_SuWT6mUIvJBITmsRIFS2tCtoNrDJhycZAC3iR05zm5TxH9fx2mhWTGnzHtv5s3Ec8AJTwH7eCdIsUfBKmJIxThc1agQ_BUSuUHX1z_QbUs0UVZxq34moqRsd2aauDUCeAdxpasmRqP5yr71AJa2yUQz03C8ftm0yFQjswt-a8gJkzRBY0ayuqRl13n-f-EZvOuCM2Oe8ZhkaF3nMc_QCkqeVZJy1g56ZwQ8uUh3-2jx3HipC0CTPSRv6m9cXViyHeK-WRIF_C8Ipfvc9l5TiwvbTtTDoDkwEKSx7jovjCD4tUSL1TdFO_tD-DNung1taQZod2LbZ3wMA8rezvadMR-1ZBk-qYI_gGLKiKg32MQ5iJzQVJ3JoVFGWO4hhwj76jOFLakyGeGIHZWUVJKva5ZgX4oKTFwVKZEwpUQtlWN_S7LyhM5SKrk4gQVhvznPvypViv0Dfse-N-jZTt-x3VG_73iuZ5u4xr7ljHpD1-o7jjO0bm4GA_do4g_dwO4NLHc4tEee5Xkj27NMDNpr2R67vvnjP5-Q2EM)](https://mermaid.live/edit#pako:eNptUk1v4jAQ_SuWT6mUIvJBITmsRIFS2tCtoNrDJhycZAC3iR05zm5TxH9fx2mhWTGnzHtv5s3Ec8AJTwH7eCdIsUfBKmJIxThc1agQ_BUSuUHX1z_QbUs0UVZxq34moqRsd2aauDUCeAdxpasmRqP5yr71AJa2yUQz03C8ftm0yFQjswt-a8gJkzRBY0ayuqRl13n-f-EZvOuCM2Oe8ZhkaF3nMc_QCkqeVZJy1g56ZwQ8uUh3-2jx3HipC0CTPSRv6m9cXViyHeK-WRIF_C8Ipfvc9l5TiwvbTtTDoDkwEKSx7jovjCD4tUSL1TdFO_tD-DNung1taQZod2LbZ3wMA8rezvadMR-1ZBk-qYI_gGLKiKg32MQ5iJzQVJ3JoVFGWO4hhwj76jOFLakyGeGIHZWUVJKva5ZgX4oKTFwVKZEwpUQtlWN_S7LyhM5SKrk4gQVhvznPvypViv0Dfse-N-jZTt-x3VG_73iuZ5u4xr7ljHpD1-o7jjO0bm4GA_do4g_dwO4NLHc4tEee5Xkj27NMDNpr2R67vvnjP5-Q2EM)
