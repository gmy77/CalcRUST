# Verifica del rilascio 2.0.0

Data: 14 ottobre 2026. Piattaforma verificata: Windows x64, Rust stabile 1.99.0.

## Controlli eseguiti

- Formattazione: `cargo fmt --check`, passato.
- Suite automatica: 13 test passati, zero fallimenti; un test di rete escluso dalla suite ordinaria.
- Cambi BCE: test live passato, dati del 2026-10-14 e 30 valute disponibili, EUR incluso.
- Compilazione release: completata con Cargo.lock.
- Avvio nativo dell'EXE finale: sette scenari conclusi con codice di uscita 0 e screenshot della sola finestra dell'app.
- Temi e viste: Standard chiara, Scientifica scura, Programmatore, Grafica, Date, Convertitore e finestra compatta 400×660.
- SHA-256 dell'EXE: contenuto in `release.json` e negli asset della release.

## Comportamenti coperti dai test

Precedenze e calcolo sequenziale Standard, parentesi, fattoriali, domini non validi, DEG/RAD/GRAD, funzioni scientifiche e iperboliche, percentuali contestuali, CE, ripetizione di uguale, EXP/F-E, radici con indice e memoria.

Aritmetica razionale: `0.1+0.2`, grandi interi, frazioni e mantenimento di `1/3` attraverso memoria e operazioni successive. Sono controllati anche gli ordini di grandezza estremi e gli esponenti non ammessi.

Programmatore: massimo u64, signed, overflow su BYTE, HEX, logica, shift, rotazioni e carry. Conversioni: andata/ritorno fra tutte le unità definite, temperatura, zero assoluto, bit e byte. Calendario: bisestili, fine mese e date non valide. Grafici: funzioni esplicite, equazioni implicite, disequazioni ed errori di funzioni sconosciute.

## Limiti

La verifica non certifica identità completa con tutte le versioni di Microsoft Calculator. Le funzioni non razionali usano f64 e i grafici sono numerici, non un motore simbolico completo. Copertura valutaria, unità e alcuni controlli grafici possono differire dall'originale. Le limitazioni numeriche sono dichiarate nel README.

Le immagini documentano il rendering; i test del motore verificano i comportamenti numerici. I sette avvii grafici non equivalgono a un collaudo manuale esaustivo di ogni interazione.
