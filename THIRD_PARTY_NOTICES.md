# Mentions tierces

RiftSwitch est distribué sous licence GPL-3.0-or-later (voir `LICENSE`). Il intègre ou adapte les travaux suivants.

## League Account Manager

La connexion par identifiants (`src-tauri/src/riot/rso_login.rs`, `captcha.rs`, `redeem.rs`) adapte la méthode et la page captcha de [League Account Manager](https://github.com/Ja-Sa-La/League-Account-Manager), sous licence MIT :

```
MIT License

Copyright (c) 2023 Ja-Sa-La

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Deceive

Le mode hors ligne (`src-tauri/src/offline.rs`) s'inspire du principe de [Deceive](https://github.com/molenzwiebel/deceive) (GPL-3.0). Aucun code n'en est repris : l'implémentation Rust est originale.

## Dépendances

Les crates Rust et paquets npm utilisés (Tauri, Svelte, reqwest, argon2, chacha20poly1305…) sont sous licences MIT et/ou Apache-2.0, compatibles avec la GPL-3.0. Leurs licences sont fournies avec leurs sources.
