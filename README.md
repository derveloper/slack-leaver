# slack-leaver

ever wanted to leave all slack channels? now you can!

## usage

```shell
git clone git@github.com:derveloper/slack-leaver.git
cd slack-leaver
cargo run
```

You need a slack user token, see [this stackoverflow answer](https://stackoverflow.com/a/67795540) on how to get one.

## limitations

Only the first 1000 channels of your workspace are considered, even if you aren't in a channel.
