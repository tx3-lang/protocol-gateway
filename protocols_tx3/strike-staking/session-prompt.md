# Estado del proyecto — Strike Finance Staking tx3

## Contexto

Proyecto: `api-layer-tx3-protocols` — Rust/Axum JSON-RPC server que carga `.tii` desde `protocols/` y los expone como métodos JSON-RPC.

El protocolo `strike-staking` ya fue investigado on-chain y reescrito. El tx3 compila y el `.tii` está desplegado.

---

## Estado actual — TODO HECHO

### Investigación on-chain ✅

Ver `investigacion/strike-staking-research.md` para el análisis completo. Resumen ejecutivo:

**Sin NFTs** — El mecanismo de credential NFTs del GitHub público NO existe en el contrato desplegado.

**Arquitectura real (dos capas)**:
- **V2Script** (`932298043...`, `addr1zxfj9xqyxcut253g30gl5tjaf9qgawp7je87e7zvduhptg275jq4yvpskgayj55xegdp30g5rfynax66r8vgn9fldndsgv4y9t`) — proxy/custodio, el usuario interactúa aquí
- **V3Script** (`1af84a9e...`, `addr1zyd0sj57d9lpu7cy9g9qdurpazqc9l4eaxk6j59nd2gkh40vvwe5f7xtt25s5fyftlm468rnjznztvgn9p0gvvr72p5qcl3cq7`) — almacén real de STRIKE stakeado
- **Batcher** (`addr1q8x4rlqhrq4rhqhnkamw3fdqmzqgum79yragg4gptcjpphmrc2rpt0exfch4s47fu32amr45vh9wg053hmcx9k7kkcrq6kxftd`) — procesa operaciones, cobra fee en STRIKE

**Reference scripts confirmados**:
- V2: `2a52d3f7be80f0e163a2fbd4fa36703e03a0d5a8139a3828e3156c335be59211#0` (5737 bytes PlutusV2)
- V3: `b3e3b7acef46f70ef511e7ea91c231a6e090a8a9790837a2b83396cf499203b3#0` (5003 bytes PlutusV2)

**Redeemers V2 (indexados)**:
- `Constr(1,[Constr(2,[])])` — 1765 ocurrencias (operación principal, batcher ruta V2→V3)
- `Constr(1,[Constr(5,[])])` — 27 ocurrencias (withdraw/distribución ADA)
- `Constr(0,[])` para mint — 1 ocurrencia (setup inicial)

**Redeemers V3 (indexados)**:
- `Constr(0,[Int, Int, Int])` — operación con cantidad, tipo e índice

**PENDIENTE**: Los redeemers exactos de `add_stake` y `withdraw_stake` desde la perspectiva del usuario (txs de Mar 2026) no estaban indexados aún. El tx3 usa los mejores candidatos disponibles.

### tx3 reescrito ✅

`strike-staking/main.tx3` — compila y pasa `trix check`. Compilado en `protocols/strike-staking.tii`.

**4 transacciones modeladas**:

| tx | Script | Plutus | Redeemer (best guess) |
|----|--------|--------|-----------------------|
| `stake` | V2Script | NO (UTxO lock simple) | — |
| `add_stake` | V2Script | SÍ | `V2Redeemer::Action { op: V2Operation::AddStake }` |
| `withdraw_stake` | V2Script | SÍ | `V2Redeemer::Action { op: V2Operation::Withdraw }` |
| `consume_rewards` | V3Script | SÍ | `V3Redeemer::Operation { redeem_amount, redeem_op: 1, redeem_idx: 0 }` |

---

## Lo que FALTA — próxima sesión

### 1. Agregar profiles al `trix.toml`

El archivo `strike-staking/trix.toml` no tiene profiles con las addresses reales. Agregar:

```toml
[profile.mainnet]
V2Script = "addr1zxfj9xqyxcut253g30gl5tjaf9qgawp7je87e7zvduhptg275jq4yvpskgayj55xegdp30g5rfynax66r8vgn9fldndsgv4y9t"
V3Script = "addr1zyd0sj57d9lpu7cy9g9qdurpazqc9l4eaxk6j59nd2gkh40vvwe5f7xtt25s5fyftlm468rnjznztvgn9p0gvvr72p5qcl3cq7"
Batcher  = "addr1q8x4rlqhrq4rhqhnkamw3fdqmzqgum79yragg4gptcjpphmrc2rpt0exfch4s47fu32amr45vh9wg053hmcx9k7kkcrq6kxftd"
staking_policy_id  = "f13ac4d66b3ee19a6aa0f2a22298737bd907cc95121662fc971b5275"
staking_asset_name = "535452494b45"
spend_script_ref   = "2a52d3f7be80f0e163a2fbd4fa36703e03a0d5a8139a3828e3156c335be59211#0"
v3_spend_script_ref = "b3e3b7acef46f70ef511e7ea91c231a6e090a8a9790837a2b83396cf499203b3#0"
```

Ver la sintaxis exacta de profiles en `trix.toml` revisando el ticketing-2026 como referencia o la documentación de trix.

### 2. Verificar los redeemers de add_stake / withdraw_stake

Las txs de Mar 2026 no estaban indexadas en Koios cuando se hizo la investigación:
- `add_stake`: `334644eca2c585c2cedc630fda259ab1c99b4db49ede39cabc5926527a8c7e76`
- `withdraw_stake`: `a7b53aebec8110b88862de53cf44862bf7064997ca8dd91885045f262e50acfc`

Verificar si ahora están indexadas:
```bash
curl -s "https://api.koios.rest/api/v1/script_redeemers?_script_hash=932298043638b552288bd1fa2e5d49408eb83e964fecf84c6f2e15a1" | \
  python3 -c "
import sys,json
data=json.load(sys.stdin)
for entry in data:
    for r in entry.get('redeemers',[]):
        if r.get('tx_hash') in [
            '334644eca2c585c2cedc630fda259ab1c99b4db49ede39cabc5926527a8c7e76',
            'a7b53aebec8110b88862de53cf44862bf7064997ca8dd91885045f262e50acfc',
        ]:
            print(r['tx_hash'][:16], r['purpose'], r.get('datum_value'))
"
```

Si los redeemers difieren de los usados en el tx3, actualizar `V2Operation` y el mapeo.

### 3. Testear contra mainnet (opcional)

Una vez con los profiles correctos, probar que el servidor JSON-RPC levanta el protocolo correctamente y que los templates de transacción son válidos.

---

## Archivos relevantes

```
investigacion/
├── main.tx3                    ← copia del tx3 actual (sincronizada)
├── strike-staking-research.md  ← investigación on-chain completa
└── session-prompt.md           ← este archivo

strike-staking/
├── main.tx3                    ← fuente del protocolo (EDITAR AQUÍ)
├── trix.toml                   ← falta agregar profiles ← PENDIENTE
└── .tx3/tii/main.tii           ← compilado local

protocols/
└── strike-staking.tii          ← binario desplegado en el servidor
```

---

## Datos clave para referencia rápida

| Concepto | Valor |
|----------|-------|
| STRIKE policy ID | `f13ac4d66b3ee19a6aa0f2a22298737bd907cc95121662fc971b5275` |
| STRIKE asset name (hex) | `535452494b45` |
| V2 script hash | `932298043638b552288bd1fa2e5d49408eb83e964fecf84c6f2e15a1` |
| V3 script hash | `1af84a9e697e1e7b042a0a06f061e88182feb9e9ada950b36a916bd5` |
| V2 reference script UTxO | `2a52d3f7be80f0e163a2fbd4fa36703e03a0d5a8139a3828e3156c335be59211#0` |
| V3 reference script UTxO | `b3e3b7acef46f70ef511e7ea91c231a6e090a8a9790837a2b83396cf499203b3#0` |
| Batcher address | `addr1q8x4rlqhrq4rhqhnkamw3fdqmzqgum79yragg4gptcjpphmrc2rpt0exfch4s47fu32amr45vh9wg053hmcx9k7kkcrq6kxftd` |
| stake tx (ejemplo) | `c49157f3896e6396aea334abb21156cf300bc388da9c86ed20a701185516f395` |
| add_stake tx (ejemplo) | `334644eca2c585c2cedc630fda259ab1c99b4db49ede39cabc5926527a8c7e76` |
| withdraw_stake tx (ejemplo) | `a7b53aebec8110b88862de53cf44862bf7064997ca8dd91885045f262e50acfc` |
| consume_rewards tx (ejemplo) | `71746ba6e072ec3cce487276bbec1005d6736475bdbb917406a341f4860fbd50` |

---

## Herramientas

```bash
# Redeemers de V2
curl -s "https://api.koios.rest/api/v1/script_redeemers?_script_hash=932298043638b552288bd1fa2e5d49408eb83e964fecf84c6f2e15a1"

# Redeemers de V3
curl -s "https://api.koios.rest/api/v1/script_redeemers?_script_hash=1af84a9e697e1e7b042a0a06f061e88182feb9e9ada950b36a916bd5"

# UTxOs de una tx
curl -s "https://api.koios.rest/api/v1/tx_utxos" \
  -H "Content-Type: application/json" \
  -d '{"_tx_hashes":["<hash>"]}'

# Info completa de una tx
curl -s "https://api.koios.rest/api/v1/tx_info" \
  -H "Content-Type: application/json" \
  -d '{"_tx_hashes":["<hash>"]}'

# Compilar y desplegar
cd strike-staking && trix check && trix build
cp .tx3/tii/main.tii ../protocols/strike-staking.tii
```
