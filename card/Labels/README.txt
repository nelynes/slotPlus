# Labels

Add a .png to GBA, GBAV, GB or GBC, named after the ROM. Labels are the same on
every device.

  Games/GBA/Metroid Fusion.gba  ->  Labels/GBA/Metroid Fusion.png

  Cart              Shape    Size
  GBA               1.94:1   1280 x 661
  Game Boy          1.13:1   1024 x 901
  Game Boy Color    1.12:1   1024 x 901

slotPlus adds these, in the same way (Ultra only for GG, NGP and TG16):

  Cart              Folder   Shown at
  WonderSwan        WS       324 x 132
  Atari Lynx        Lynx     284 x 190
  Game Gear         GG       290 x 200
  NeoGeo Pocket     NGP      246 x 133
  TurboGrafx-16     TG16     191 wide, up to 163 tall

Other sizes are scaled to fill the label and cropped from the centre, except
on a HuCard (TG16), where the art keeps its own shape and sits at the bottom.
Cart Studio makes these: https://studio.slot-cfw.fyi
Cart Studio+ makes them for every slotPlus system: https://nelynes.github.io/slotPlus-studio/


# Shells

cart_shell.txt sets a cart's shell, one line per game:

  Metroid Fusion = auto 6a3fa0 solid

  Outline   auto, notched or rounded
  Colour    hex RGB
  Finish    solid, clear or glitter
