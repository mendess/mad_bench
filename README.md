# Mad Bench

I'm trying to benchmark 3 iterators + "manual iteration". But I'm getting a
weird phenomena. Changing code on one of the iterators changes the performance of
the others despite them being in no way related. This has happened before,
changing a function affects the performance of another.

(more context: This is a bit vector, what it does isn't too important I think
but it stores numbers using less than a byte for each one. Hence the code is
full of bit magic)

The iterator being changed is [bit_array::Iter][iter]

## Version 1

[commit](https://github.com/mendess/mad_bench/tree/7022e96a15e3b6ab636295b0a7cb08baae340014)

The benchmark at [benches/bit_array.rs](benches/bit_array.rs) yielded the
following results:

<img src='https://raw.githubusercontent.com/mendess/mad_bench/master/version1.svg'/>

## Version 2

[commit](https://github.com/mendess/mad_bench/tree/b742caa758765f0f3680386e598b254889ff5e34)

The first version to me seemed sub optimal, so I made this change:

[diff](https://github.com/mendess/mad_bench/commit/b742caa758765f0f3680386e598b254889ff5e34)

And once again ran the same benchmark, which showed that my change wasn't
better, despite making less checks... but at the same time the other iterators
got faster.

<img src='https://raw.githubusercontent.com/mendess/mad_bench/master/version2.svg'/>

As I said this has happened to me before and I have no idea how to find
troubleshoot mistake.

I ran the benchmarks several times on both commits to test if it was noise but
they were very consistent.

## Running
```sh
git checkout 7022e96
cargo bench
cp -r target/criterion/ ver1
git checkout b742caa
cargo bench
cp -r target/criterion/ ver2
git checkout master
```

To see them:

- `$BROWSER ver1/report/index.html`
- `$BROWSER ver2/report/index.html`

[iter]: https://github.com/mendess/mad_bench/blob/b742caa758765f0f3680386e598b254889ff5e34/src/bit_array.rs#L110
[v1]: https://raw.githubusercontent.com/mendess/mad_bench/master/version1.svg
[v2]: https://raw.githubusercontent.com/mendess/mad_bench/master/version2.svg
