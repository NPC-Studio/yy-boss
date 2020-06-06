# YY-BOSS

This is a library, created for the development of [Fields of Mistria](https://twitter.com/FieldsofMistria), a farming RPG with *tons* of Sprites, by NPC Studio. [Jack Spira](https://twitter.com/sanbox_irl) wrote the first version of this tool and maintains it. This tool was created to support an Aseprite -> GMS2 pipeline tool. That tool is not public, but largely because it's specific to our own workflows. Using this tool, one should be able to generate their own pipeline without difficulty.

**This crate only supports Gms2 and the upcoming 2.3 release. 2.2 is NOT supported.**

## YY-Boss

This repository is dedicated to the YY-BOSS, a higher level interface for Gms2. What is a higher level interface? In this context, the YY-BOSS can be used by a user with absolutely no knowledge of Gms2. It has an entirely walled off Api (though there are getters for external debugging). Common usage, for example, would be to make a YypBoss and then ask it to create a Sprite with such and such name and such and such pngs. The YypBoss will handle it from there.

If a lower level control is something you're interested in, the typings are entirely public facing, and you can use those to build your own thing. If, on the the other hand, you are more interested in building tools using a YYP manipulator, then this might suit your needs.

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
- [X] Included Files
- [ ] Extensions
- [X] Options
- [X] Configurations

## The Future Development of this Crate

This crate will be developed as it is needed for further tools for *Fields of Mistria*. Eventually, even if *Fields of Mistria* does not need it, the intention is for this crate to be feature complete.

If a user requires typings for the `yy` files immediately, and is not prepared to wait for this crates completion, but finds themselves overwhelmed by the `yy` files, check out my previous Typescript repository. It is slightly out of date, and some of the comments are incorrect, but it is [available here](https://github.com/sanbox-irl/yyp-typings).
