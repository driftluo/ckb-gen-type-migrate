# ckb-gen-type-migrate

The `ckb-gen-types` crate had a major breaking change during the upgrade to 0.119, which included the following improvements:

1. Add `From/Into` implementation for all structures while retaining `Pack/UnPack`
2. Remove the use of `Pack/Unpack` in all libraries and change them all to `From/Into`
3. Implement generics for molecule builder function

This change is mainly to improve the usability of the API so that users no longer have to worry about whether to use pack or unpack. However, the change will also break some code. 

This library is used to solve more than 90% of migration code problems

### Usage

First, upgrade your project's `ckb-gen-type` to `0.119`, and then use the following operation

There are two modes of use

1. Call cargo yourself in the project. Please note that cargo error messages are hierarchical and you need to call cargo repeatedly to maximize the effect. Recommended call 10 times

```bash
$ cargo c --tests --message-format json | ckb-gen-type-migrate
```

2. ckb-gen-type-migrate automatically calls cargo to perform operations, the default is to run 10 times

```bash
$ ckb-gen-type-migrate --cargo
```

After that, most of the problems have been solved
