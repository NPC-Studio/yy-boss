# YY-BOSS

This is a library, created for the development of [Fields of Mistria](https://twitter.com/FieldsofMistria), a farming RPG with *tons* of Sprites, by NPC Studio. [Jack Spira](https://twitter.com/sanbox_irl) wrote the first version of this tool and maintains it. This tool was created to support an Aseprite -> GMS2 pipeline tool. That tool is not public, but largely because it's specific to our own workflows. Using this tool, one should be able to generate their own pipeline without difficulty.

This repository is divided into two halves, neither of which are wide enough to cover GMS2, but both of which are deep enough for GMS2 so far.

## YY-Typings

The first half of this crate are typings for the `.yyp` and various `.YY` files. The following typings have been created (and are actively supported):

- [x] YYP
- [x] Sprites
- [x] ResourceTypes
- [x] TextureGroups
- [ ] Tilesets
- [ ] Sounds
- [ ] Paths
- [ ] Scripts
- [ ] Shaders
- [ ] Fonts
- [ ] Timelines
- [ ] Objects
- [ ] Rooms
- [ ] Notes
- [ ] Included Files
- [ ] Extensions
- [x] Options*
- [ ] Configurations

***Options are not directly user accessible, but are provided through other structures like TextureGroups. The Options file itself is very non-spec, and it difficult to get a cohesive picture of.**

## YY-Boss

The second half of this crate, which is entirely optional to use via extension traits, is a higher level interface for buildings, managing, and destroying resources in the YYP. It includes full control over `yy` files and their associated data. It is opinionated, and it demands certain crates be used in conjunction with it (such as `image` for processing `png` files).

If a lower level control is something you're interested in, the typings are entirely public facing, and you can use those to build your own thing. If, on the the other hand, you are more interested in building tools using a YYP manipulator, than this might suit your needs.

The following has been done:

- [x] YYP
- [x] Sprites
- [x] ResourceTypes
- [x] TextureGroups
- [ ] Tilesets
- [ ] Sounds
- [ ] Paths
- [ ] Scripts
- [ ] Shaders
- [ ] Fonts
- [ ] Timelines
- [ ] Objects
- [ ] Rooms
- [ ] Notes
- [ ] Included Files
- [ ] Extensions
- [ ] Options
- [ ] Configurations

## The Future Development of this Crate

This crate will be developed as it is needed for further tools for *Fields of Mistria*. Eventually, even if *Fields of Mistria* does not need it, the intention is for this crate to be feature complete.

Equally, the maintainer will have this crate ready for GMS2 2.3 on release day, or shortly thereafter.

If a user requires typings for the `yy` files immediately, and is not prepared to wait for this crates completion, but finds themselves overwhelmed by the `yy` files, check out my previous Typescript repository. It is slightly out of date, and some of the comments are incorrect, but it is [available here](https://github.com/sanbox-irl/yyp-typings).
