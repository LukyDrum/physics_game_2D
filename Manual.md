English version bellow

# Manuál

Vítejte ve 2D fyzikální simulaci.
Vaším cílem v této hře je experimentovat s fyzikou tekutin a tuhých těles.
Po zapnutí hry se ocitnete rovnou v hlavní herní scéně.
- Po levé straně vidíte pomocí černého čtverce vyznačenou herní plochu
- V prostřední části obrazovky vidíte výběr nástrojů
- Aktuálně vybraný nástroj je vyznačen černým okrajem
- Nástroje jsou:
    - "Info" - zobrazuje informace o simulaci
    - "Fluids" - nástroj pro tvorbu a konfiguraci tekutin
    - "Bodies" - nástroj pro tvorbu a konfigurtaci těles
    - "Config" - umožní vám upravit parametry fyzikálních simulací
    - "Saves/Loads" - umožní vám uložit či načíst úrovně
- Po pravé straně obrazovky se nachází "Quick Menu" obsahující rychlé akce pro pozastavení simulace, restartování současné úrovně a opuštení programu

## Nástroj Fluids

- Po vybrání toho nástroje můžete v herní ploše podržením levého tlačítka myši vytvářet tekutinu
- V prostřední části obrazovky také získáte možnost nastavit hustotu (v g/cm^3) a barvu (v RGB formátu) tekutiny - možnosti "Density" a "Color"
- Dále máte možnost nastavit kolik "kapek" tekutiny bude vytvořeno na jeden klik - možnost "Droplet Count"
- Červeným tlačítkem "Clear fluid" odstraníte veškerou tekutiny z herní plochy

## Nástroj Bodies

- Po vybrání toho nástroje můžete v herní ploše kliknutím pravého tlačítka myši vytvářet tělesa
- Pomocí prostředního tlačíka myši (zmáčknutí kolečka) vymažete těleso pod kurzorem myši z herní plochy
- Pomocí podržení levého tlačíka myši může chytit a přesouvat tělesa ve scéně
- V současné době jsou všechna tělesa pouze obdelníky
- V prostřední části obrazovky opět máte následující nastavení:
    - "Width" - šířka tělesa
    - "Height" - výška tělesa
    - "Orientation" - otočení tělesa ve stupních
    - "Mass" - hmotnost tělesa v gramech
    - "Is Static?" - zaškrtnutím toto těleso označíte jako statické a zamezíte mu tak v pohybu
    - "Elasticity" - elastičnost kolize
    - "Static Friction" - koeficient statické tření
    - "Dynamic Friction" - koeficient dynamického tření
    - "Color" - barva tělesa ve formátu RGB

## Nástroj Saves/Loads

- Pro uložení současného stavu herní plochy změňte název v políčku textu na libovolný název a klikněte na tlačítko "Save"
- Pro načtení úrovně klikněte na libovolný název úrovně ze seznamu
