# My digital garden project

So, this is the repo for my website. The code is pretty much self-explainatory
(most of the time) but there are some tricky stuff there (especially client side
things like using D3 and so on)

## Prerequesites

In order to use or even run this project locally you would either need
TailwindCSS cli installed (through NPM or whatever) or Nix (the package manager)
and direnv installed.

The easiest way is to have Tailwind installed through NPM. The cooler way is to
use nix flakes and direnv to setup the dev environment automatically. Either
way, once that is there you can just

```sh
yourcooldesktop@DESKTOP:~$ bacon dev
```

And that's it my friend. The server is up and running. The site is accesible in
[http://localhost:3000](http://localhost:3000/)

## How it works

So, what you have running right now is an [actix web](https://actix.rs/) process
serving some static HTML with a bit of JS. It spins up a
[SurrealDB](https://surrealdb.com/) in-memory instance in which we store all of
the posts (and other stuff). The posts are inside the
[/garden](https://github.com/Chemchu/guslee/tree/main/garden) directory. We do
all sort of operations with those documents like creating relationships between
posts, creating a full-search engine utility crate (relying on surrealdb tbh)
and more stuff I can't remember right now.

It's important to note that the surrealdb server is only accessible on
development. In production, the in-memory database is not exposed.

There is also a special file called
[build.rs](https://github.com/Chemchu/guslee/blob/main/build.rs) in this project
which runs each time a file is saved. When that happens, it then recompiles all
the CSS used here. This is why the tailwind cli is needed, otherwise an error
will pop up and the styles are going to be messed up.

## Hacky solutions

### Tailwind Prose

There are some css class add/remove for the main view. The "prose" css class is
needed to render the markdown post but keeping it for non-post pages kinda
breaks the site. That's why I had to add and remove the class depending on the
current url. Not the cleanest solution but still a solution soooo shut up
