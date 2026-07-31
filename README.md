# Nenechi CLI

Colección de utilidades para organizar la biblioteca multimedia del servidor
Nenechi.

## Instalar

Descarga el paquete `.deb` para tu arquitectura (`amd64` o `arm64`) desde la
[última versión publicada](https://github.com/magonxesp/nenechi-cli/releases/latest).
Este enlace redirige automáticamente a la release más reciente.

```bash
sudo dpkg -i ./nenechi_*.deb
```

Comprueba la instalación con:

```bash
nenechi-cli --help
```

## Cómo usarlo

La aplicación agrupa sus utilidades en dos comandos:

| Comando | Descripción |
| --- | --- |
| `jellyfin` | Indexa y organiza series y películas para Jellyfin. |
| `wallpapers` | Indexa y clasifica fondos de pantalla. |

Puedes consultar los subcomandos disponibles en cualquier nivel:

```bash
nenechi-cli --help
nenechi-cli jellyfin --help
nenechi-cli wallpapers --help
```

### Jellyfin

Configura primero los directorios de origen y destino siguiendo el
[ejemplo de configuración de Jellyfin](examples/conf.d/jellyfin.yaml).
Si alguno de los targets de series tiene la categoría `anime`, añade también
un Client ID de la API v2 de MyAnimeList al
[fichero de configuración general](examples/config.yaml).

```bash
# Indexa las series y obtiene sus metadatos.
nenechi-cli jellyfin index

# Crea en los destinos la estructura de Jellyfin mediante enlaces simbólicos.
nenechi-cli jellyfin mount
```

`mount` actualiza primero el índice de las series que todavía no estén
registradas. Los nombres creados en el destino se normalizan para que sean
compatibles con clientes Windows a través de SMB, sin modificar los ficheros
originales ni los títulos almacenados en SQLite.

### Wallpapers

Configura los directorios, patrones ignorados y destinos de clasificación
siguiendo el
[ejemplo de configuración de wallpapers](examples/conf.d/wallpapers.yaml).

```bash
# Guarda en SQLite los metadatos de las imágenes encontradas.
nenechi-cli wallpapers index

# Clasifica las imágenes mediante enlaces simbólicos por formato y tags.
nenechi-cli wallpapers tidy
```

Cuando el nombre de una imagen contiene un identificador de Pixiv, el
indexado intenta recuperar también sus tags. `tidy` indexa automáticamente
las imágenes que todavía no estén registradas. El subcomando `clean-index`
aparece en la ayuda, pero aún no está implementado.

## Configuración

La configuración está dividida entre el fichero general
[`config.yaml`](examples/config.yaml) y un fichero por comando dentro de
[`conf.d`](examples/conf.d). Copia los ejemplos a una de estas ubicaciones,
que se comprueban en este orden:

1. `~/.nenechi`
2. `~/.config/nenechi`
3. `/etc/nenechi`

Por ejemplo, para una instalación de usuario:

```bash
mkdir -p ~/.config/nenechi/conf.d
cp /usr/share/nenechi/config.yaml ~/.config/nenechi/config.yaml
cp /usr/share/nenechi/conf.d/*.yaml ~/.config/nenechi/conf.d/
```

Si los ejemplos no están disponibles en `/usr/share/nenechi`, puedes
descargarlos desde el directorio
[`examples`](https://github.com/magonxesp/nenechi-cli/tree/main/examples) de
la rama `main` del repositorio.

Antes de ejecutar los comandos, adapta las rutas, credenciales y permisos de
los ejemplos a tu entorno. La referencia de cada comando se mantiene en su
propio fichero: [Jellyfin](examples/conf.d/jellyfin.yaml) y
[wallpapers](examples/conf.d/wallpapers.yaml).

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

## Probar Debian desde macOS

Los servicios de `docker-compose.yml` permiten generar el paquete Linux,
instalarlo en un Debian desechable y probar la imagen de la aplicación desde
macOS u otro sistema con Docker Compose.

Prepara la configuración compartida por los contenedores:

```bash
make sandbox
```

Los ejemplos se copian a `sandbox/home/.config/nenechi`. Puedes editar esas
copias para usar rutas y credenciales de prueba; las siguientes ejecuciones de
`make sandbox` no sobrescriben tus cambios.

Genera el paquete Debian:

```bash
docker compose run --rm packager
```

El paquete se guarda en `target/release/bundle` y usa la arquitectura del
contenedor. Instálalo y comprueba el ejecutable en un Debian limpio con:

```bash
docker compose run --rm sandbox bash -lc \
  'dpkg --install target/release/bundle/nenechi_*.deb && nenechi --help'
```

Para probar directamente la imagen definida en `Dockerfile`:

```bash
docker compose run --rm nenechi_cli --help
```
