<p align="center">
  <img src="assets/logo.png" alt="Wallpaper Manager logo" width="300"/>
</p>

# Wallpaper Manager

Um gerenciador de wallpapers para Hyprland com suporte a múltiplos monitores, interface gráfica (GUI) e restauração automática do fundo após reinício.

---

## Funcionalidades

- Interface gráfica com seleção de imagens
- Suporte a múltiplos monitores (`hyprctl`)
- Integração com `swww`
- Salvamento da última seleção de wallpaper
- Subcomando `apply` para reaplicar automaticamente após reinício

---

## Requisitos

- Sessão Wayland (Hyprland, Sway, etc.)
- [`swww`](https://github.com/LabDump/swww)
- `hyprctl` (Hyprland instalado)
- Bibliotecas de sistema:
  - `wayland`, `libxkbcommon`, `libX11`, `libGL`, `vulkan-loader`

---

## Instalação

### Via Nix (recomendado)

```bash
nix run github:z4nder/wallpaper-manager -- gui
# ou
nix profile install github:z4nder/wallpaper-manager
```

### Via Cargo

```bash
cargo install wallpaper-manager
```

> Certifique-se de ter as bibliotecas Wayland e o `swww` instalados no sistema.

---

## Como usar

### Interface gráfica

```bash
wallpaper-manager gui
```

> Navegue pelas imagens em `~/.config/backgrounds` e aplique com um clique.

### Aplicar último wallpaper salvo

```bash
wallpaper-manager apply
```

---

## Execução automática no login

Adicione ao seu `~/.config/hypr/hyprland.conf`:

```ini
exec-once = swww-daemon
exec-once = wallpaper-manager apply
```

> Isso garante que o daemon `swww` será iniciado e o wallpaper restaurado ao login.

---

## Organização dos arquivos

Por padrão, o programa busca imagens em:

```
~/.config/backgrounds
```

Coloque seus wallpapers nesta pasta. Exemplo:

- `~/.config/backgrounds/meadow.jpg`
- `~/.config/backgrounds/dark-moon.png`

---

## Contribuindo

Contribuições são bem-vindas!

- Relate problemas
- Envie pull requests
- Sugira melhorias

---

## Licença

MIT © 2025 - Z4nder
