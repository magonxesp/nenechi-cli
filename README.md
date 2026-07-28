# Nenechi cli

Comandos utiles para el servidor 😎

## Instalar

## Como usarlo

## Configuración

Los ficheros de ejemplo se encuentran en el directorio
[`examples`](https://github.com/magonxesp/nenechi-cli/tree/main/examples) del
repositorio. Descárgalos con:

```bash
git clone --depth 1 https://github.com/magonxesp/nenechi-cli.git
cd nenechi-cli
```

Después, copia `examples/config.yaml` y el directorio `examples/conf.d` en una
de las siguientes ubicaciones. El programa las comprueba en este orden:

1. `~/.nenechi`
2. `~/.config/nenechi`
3. `/etc/nenechi`

Por ejemplo, para usar `~/.nenechi`:

```bash
mkdir -p ~/.nenechi/conf.d
cp examples/config.yaml ~/.nenechi/config.yaml
cp examples/conf.d/wallpapers.yaml ~/.nenechi/conf.d/wallpapers.yaml
```

Para usar `~/.config/nenechi`:

```bash
mkdir -p ~/.config/nenechi/conf.d
cp examples/config.yaml ~/.config/nenechi/config.yaml
cp examples/conf.d/wallpapers.yaml ~/.config/nenechi/conf.d/wallpapers.yaml
```

Para instalar la configuración global en `/etc/nenechi`:

```bash
sudo mkdir -p /etc/nenechi/conf.d
sudo cp examples/config.yaml /etc/nenechi/config.yaml
sudo cp examples/conf.d/wallpapers.yaml /etc/nenechi/conf.d/wallpapers.yaml
```

Para indexar targets de Jellyfin con categoría `anime`, configura el Client ID
de la API v2 de MyAnimeList en `config.yaml`:

```yaml
myanimelist:
  api_key: your-client-id
```

Después ejecuta:

```bash
nenechi-cli jellyfin index
```

Para crear en cada `destination` la estructura que espera Jellyfin y enlazar
los ficheros del `source`, ejecuta:

```bash
nenechi-cli jellyfin mount
```

El comando consulta primero el índice de series y añade automáticamente las
que todavía no estén indexadas.

## Testing

Las pruebas de integración de MyAnimeList necesitan una API key. Crea un
fichero `.env` en la raíz del repositorio con este contenido:

```dotenv
MYANIMELIST_API_KEY=tu_api_key
```

El fichero `.env` está ignorado por Git y se carga únicamente desde las
pruebas. Para ejecutar todos los tests:

```bash
cargo test --workspace
```
