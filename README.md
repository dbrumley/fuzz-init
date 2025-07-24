# mayhem-init

The easiest way to get started integrating fuzzing into your app is by using `mayhem-init`.
This CLI tool enables you to quickly start building the proper scaffolding and new fuzz harness, with everything set up for you to run in Mayhem. You can also create a new skeleton template for a new app that includes fuzzing and follows best practices.

To get started, simply run:

```
mayhem-init
```

To create scaffolding in a specific folder, you can send a name as an argument. For example, the following command will create new fuzz scaffolding in the `my-app` folder:

```
mayhem-init my-app
```

## Options

`mayhem-init` comes with the following options:

- `--language` - Select a particular language from the template list.
