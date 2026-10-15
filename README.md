# Ar Condicionado

App de bandeja do Windows para controlar ar-condicionado Gree (e marcas com o mesmo módulo Wi-Fi) pela rede local, via o protocolo UDP do Gree, sem nuvem. Tauri 2 + React + TypeScript + Tailwind.

## Requisitos

- Windows 10/11 com WebView2
- Node 20+ e Rust (stable, MSVC)
- PC e aparelho na mesma rede; o Firewall do Windows pode pedir permissão na primeira busca

## Desenvolvimento

```sh
npm install
npm run tauri dev
```

Build do instalador (NSIS em `src-tauri/target/release/bundle/`):

```sh
npm run dist   # ou: npm run tauri build
```

## Testando o protocolo com o aparelho real

Antes de usar a UI, dá para validar scan, bind e status direto no terminal:

```sh
cd src-tauri
cargo run --example gree_debug            # primeiro aparelho encontrado
cargo run --example gree_debug f4911e...  # um MAC específico
cargo test                                # vetores de criptografia e offset do TemSen
```

Só lê o estado; nenhum comando é enviado ao aparelho.

## Como funciona

- Busca: broadcast `{"t":"scan"}` na porta 7000 (global e por interface), respostas coletadas por 3s.
- Criptografia: v1 (AES-128-ECB) e v2 (AES-128-GCM), detectadas automaticamente no scan e no bind.
- Bind com a chave genérica; a chave do aparelho é salva em `%APPDATA%\com.local.arcondicionado\config.json` junto com IP, MAC e versão.
- Timeout de 2s por pacote com 2 tentativas. Se o aparelho parar de responder, uma nova busca pelo MAC atualiza o IP (DHCP).
- Status a cada 5s com o flyout aberto e a cada 60s fechado; o tooltip da bandeja acompanha.

Referências do protocolo: [greeclimate](https://github.com/cmroche/greeclimate) e [gree-remote](https://github.com/tomikaa87/gree-remote).
