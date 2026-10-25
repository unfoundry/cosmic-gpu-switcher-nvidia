# cosmic-gpu-switcher-nvidia
An (Unofficial) Cosmic Applet to switch Graphics Modes on Devices with Nvidia Graphics Cards.

<img width="873" height="669" alt="image" src="https://github.com/user-attachments/assets/ae1875a9-b469-455b-9dac-4aa4d0faa64b" />

### Why?
Cosmic Desktop is awesome especially the new Cosmic Epoch that now uses Rust inside! As it's quite in its early days of development, I couldn't find a GUI toggle to switch my Graphics modes easily (There is an easy cli command that does this for you though, but I thought I'd give this a shot)

### How?
This simply fires the already built and stable `system76-power graphics` command and presents the options to you.
The following options are what suited me best:
- `integrated`
- `compute`
- `hybrid`
- `nvidia`

### Who?
I'm a total newbie to Rust, would like this to be a first step (so this definitely includes AI) and hence has no intention to merge this to become a real applet unless a proper developer takes a look at this and makes it more efficient.

#### DISCLAIMER
This project includes an SVG that denotes the **NVIDIA** logo and hence all rights to the logo belongs **NVIDIA**. Also mentions about **Cosmic** which is part of **System76**'s awesome software offerings. This is purely developed in good faith.
