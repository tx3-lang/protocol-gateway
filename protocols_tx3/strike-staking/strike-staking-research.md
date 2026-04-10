# Strike Finance Staking — Investigación On-Chain

## Identificadores del protocolo

| Campo | Valor |
|-------|-------|
| STRIKE policy ID | `f13ac4d66b3ee19a6aa0f2a22298737bd907cc95121662fc971b5275` |
| STRIKE asset name (hex) | `535452494b45` ("STRIKE") |
| STRIKE decimales | 6 |
| STRIKE minting tx | `d11ca3dd03e899fbd1d76da68dac619617dd56372f65ef55e68ced1334f38e08` (block 10755594) |

---

## ⚠️ Contrato activo (correcto)

> **El contrato activo con mayor volumen NO es v2 ni v3 de las secciones anteriores.**
> El tx3 basado en el GitHub público (bkp_main.tx3) apunta al contrato CORRECTO.
> La investigación de v2/v3 abajo describe contratos legacy con menor actividad.

| Campo | Valor |
|-------|-------|
| Script hash | `497a8b0085517f1c9065cf3006af4c266454b39c6fa32a9d116c75ee` |
| Tipo | PlutusV3 (3781 bytes) |
| Dirección de staking | `addr1z9yh4zcqs4gh78ysvh8nqp40fsnxg49nn3h6x25az9k8tms6409492020k6xml8uvwn34wrexagjh5fsk5xk96jyxk2qf3a7kj` |
| Reference script UTxO | `486c6c010d1518b1e032d2a288483fba55cee4f054b6e97f4e7eadeccb173768#0` (spend Y mint = mismo UTxO) |
| mint_policy_id | `497a8b0085517f1c9065cf3006af4c266454b39c6fa32a9d116c75ee` (= script hash) |
| tracker_asset_name | `535452494b45` (= mismo que STRIKE asset name) |
| Actividad | 658+ txs desde block 13050000, ~41k redeemers indexados |

### Redeemers confirmados on-chain (contrato activo)

| Operación | Tipo | Estructura on-chain | Ocurrencias |
|-----------|------|---------------------|-------------|
| `stake` | mint | `Constr(0, [Int])` — amount | 2456 |
| `withdraw_stake` | mint | `Constr(1, [Bytes])` — owner_pkh | 1303 |
| `add_stake` / `consume_rewards` | spend | `Constr(0, [])` | 15764 |
| `withdraw_stake` | spend | `Constr(1, [])` | 1307 |
| `distribute_rewards` | spend | `Constr(2, [])` | 18903 |

Esto coincide **exactamente** con el código del GitHub público:
- `MintRedeemer::Mint { amount }` = mint Constr(0)
- `MintRedeemer::Burn { owner_address_hash }` = mint Constr(1)
- `StakingRedeemer::AddStakeOrConsumeStakingRewards` = spend Constr(0)
- `StakingRedeemer::WithdrawStake` = spend Constr(1)
- `StakingRedeemer::DistributeStakingRewards` = spend Constr(2)

### Tx hashes de referencia (más recientes por operación)

| Operación | Tx hash | Block |
|-----------|---------|-------|
| `stake` | `f70239fa91bcb0df496c011465e440e8cd97955231df956c7de3820e1c861a80` | 13140934 |
| `add_stake` | `939737ecd4f1ed2cab613a63606802287e79b0525767ddaff6057bbafd28bfab` | 13140518 |
| `withdraw_stake` | `60f83cf13c421d33a039cd902b57dd67fec4f4afbcdbade5a597b3f816d58b06` | 13140491 |
| `consume_rewards` | `4a458236a0b4074703970fffa32a337314094963df6255e07dd967a690466260` | 13139985 |

---

## Contratos legacy (menor actividad)

El protocolo ha tenido **3 versiones previas** del contrato de staking:

| Versión | Script hash | Dirección | Período activo |
|---------|-------------|-----------|----------------|
| v1 | `2025463437ee5d64e89814a66ce7f98cb184a66ae85a2fbbfd750106` | `addr1zysz2335xlh96e8gnq22vm88lxxtrp9xdt595taml46szpnqcef7gz6hguyxwz0wuwuq64ryws4ws8pqennhd28rgh8s0nh7pw` | Ago 2024 (pocos días) |
| v2 | `932298043638b552288bd1fa2e5d49408eb83e964fecf84c6f2e15a1` | `addr1zxfj9xqyxcut253g30gl5tjaf9qgawp7je87e7zvduhptg275jq4yvpskgayj55xegdp30g5rfynax66r8vgn9fldndsgv4y9t` | Ene 2025 – actual |
| v3 | `1af84a9e697e1e7b042a0a06f061e88182feb9e9ada950b36a916bd5` | `addr1zyd0sj57d9lpu7cy9g9qdurpazqc9l4eaxk6j59nd2gkh40vvwe5f7xtt25s5fyftlm468rnjznztvgn9p0gvvr72p5qcl3cq7` | Ago 2024 – actual |

---

## Reference Scripts (HALLAZGO CLAVE)

| Contrato | UTxO del reference script | Block creación | Tamaño script |
|----------|--------------------------|---------------|---------------|
| v2 | `2a52d3f7be80f0e163a2fbd4fa36703e03a0d5a8139a3828e3156c335be59211#0` | 10771553 | 5737 bytes (PlutusV2) |
| v3 | `b3e3b7acef46f70ef511e7ea91c231a6e090a8a9790837a2b83396cf499203b3#0` | (ver nota) | 5003 bytes (PlutusV2) |

**Nota**: Los reference scripts están almacenados *dentro* de las propias direcciones de script (v2 guarda el ref en `addr1zxfj9...`, v3 en una variante con diferente staking credential pero mismo payment hash `1af84a9e...`).

**Confirmado via tx_size**: Las txs de `add_stake` (1294 bytes) y `consume_rewards` (1363 bytes) son demasiado pequeñas para incluir los scripts inline (5737 y 5003 bytes). Por lo tanto sí usan reference scripts. La API Koios no los muestra en `reference_inputs` (bug de indexación).

---

## ¡NO HAY NFTs! — Hallazgo crítico

**CONFIRMADO**: El mecanismo de credential NFTs (tracker token + owner NFT) descrito en el GitHub público y modelado en `main.tx3` **NO está implementado en el contrato desplegado**.

Evidencia:
- `assets_minted: []` en TODAS las txs de staking analizadas
- La tx `stake` (c49157f3...) **no tiene ejecución Plutus** (sin colateral) — es solo un envío de UTxO a la dirección del script
- Las txs `add_stake` y `withdraw_stake` ejecutan Plutus (tienen colateral) pero sin minting/burning
- El contrato desplegado NO tiene un parámetro `mint_policy_id` separado del script hash

---

## Transacciones ejemplo por operación

### `stake` — Primer staking del protocolo

**TX**: `c49157f3896e6396aea334abb21156cf300bc388da9c86ed20a701185516f395`
**Bloque**: 10762931 (Ago 2024) — contrato v1
**¡Sin ejecución Plutus!** — tx_size 976B, sin colateral, `plutus_contracts: []`

```
INPUTS:
  [wallet] 160,644,244 lovelace
  [wallet] 5,448,879 lovelace + 42,850,000 STRIKE

OUTPUTS:
  [wallet] 1,000,000 lovelace  (fee al batcher?)
  [wallet] 161,867,638 lovelace + 32,850,000 STRIKE  (cambio)
  [SCRIPT v1] 3,000,000 lovelace + 10,000,000 STRIKE  ← UTxO de staking con datum inline
```

**Patrón**: plain UTxO lock, sin validador, sin redeemer, sin NFTs.

---

### `add_stake` — Agrega STRIKE a posición existente

**TX**: `334644eca2c585c2cedc630fda259ab1c99b4db49ede39cabc5926527a8c7e76`
**Bloque**: 13028149 (Mar 2026) — contrato v2
**Ejecuta Plutus** (colateral presente), tx_size 1294B → usa reference script

```
INPUTS (Koios incompleto — falta el script input):
  [wallet] 11,412,898 lovelace + 2,558,636 STRIKE
  [wallet] 9,948,452 lovelace
  [SCRIPT v2] (input existente, no mostrado por Koios)

OUTPUTS:
  [addr1q8x4...] 6,000,000 lovelace  (batcher/fee address, con datum)
  [wallet]       3,454,877 lovelace  (cambio)
  [wallet]       5,184,376 lovelace + 128,636 STRIKE  (cambio)
  [SCRIPT v2]    2,025,192 lovelace + 2,430,000 STRIKE  ← UTxO de staking actualizado
  [SCRIPT v3]    4,349,622 lovelace  (pago a contrato v3/treasury?)
```

---

### `withdraw_stake` — Retiro completo

**TX**: `a7b53aebec8110b88862de53cf44862bf7064997ca8dd91885045f262e50acfc`
**Bloque**: 13028259 (Mar 2026) — contrato v2
**Ejecuta Plutus**, tx_size similar → usa reference script

```
INPUTS:
  [wallet]    22,807,219 lovelace + 128,636 STRIKE
  [SCRIPT v2] 4,770,257 lovelace + 3,694,342 STRIKE  ← UTxO de staking

OUTPUTS:
  [addr1q8x4...] 6,000,000 lovelace + 147,773 STRIKE  (batcher fee en STRIKE)
  [wallet]       6,000,000 lovelace + 3,546,569 STRIKE
  [wallet]       3,509,593 lovelace + 128,636 STRIKE
  [SCRIPT v2]    11,749,354 lovelace  (ADA residual al contrato)
```

STRIKE total: 3,694,342 + 128,636 = 3,822,978 IN → 147,773 + 3,546,569 + 128,636 = 3,822,978 OUT ✓

---

### `consume_rewards` — Reclamar rewards sin retirar stake

**TX**: `71746ba6e072ec3cce487276bbec1005d6736475bdbb917406a341f4860fbd50`
**Bloque**: 13002691 (Mar 2026) — contrato v3
**Ejecuta Plutus**, tx_size 1363B → usa reference script

```
INPUTS:
  [wallet]    30,000,000 × 3 + 10,000,000 lovelace  (ADA del batcher para rewards)
  [SCRIPT v3] 2,000,000 lovelace + 121,738,565 STRIKE

OUTPUTS:
  [wallet] 6,000,000 lovelace + 116,550 STRIKE  (batcher fee)
  [wallet] 6,000,000 lovelace + 2,797,202 STRIKE  (rewards al usuario)
  [wallet] 82,597,686 lovelace  (ADA devuelta al batcher)
  [wallet] 5,035,000 lovelace  (ADA devuelta)
  [SCRIPT v3] 2,000,000 lovelace + 118,824,813 STRIKE  ← posición actualizada
```

STRIKE: 121,738,565 → 118,824,813 + 2,913,752 (rewards) ✓

---

## Estructura de redeemers (contrato desplegado)

### v2 — Redeemers de spend

```
Constr(1, [Constr(2, [])])  — 1765 ocurrencias: operación principal (ruta STRIKE v2→v3)
Constr(1, [Constr(3, [])])  — 2 ocurrencias: depósito ADA al pool
Constr(1, [Constr(5, [])])  — 27 ocurrencias: retiro/distribución masiva de ADA
```

### v2 — Redeemer de mint

```
Constr(0, [])  — 1 ocurrencia: creación inicial del pool
```

### v3 — Redeemers de spend

```
Constr(0, [Int, Int, Int])  — operación con parámetros de cantidad e índice
Ejemplos: Constr(0,[120000000,0,0]), Constr(0,[12000000000,1,0])
```

**Nota**: Los redeemers del contrato desplegado son completamente distintos a los del `main.tx3` basado en el GitHub público.

---

## Arquitectura real del protocolo

```
┌─────────────────────────────────────────────────────┐
│                 Strike Finance DEX/Staking           │
│                                                     │
│   v3 (1af84a9e...)          v2 (93229804...)        │
│   addr1zyd0...              addr1zxfj9...           │
│                                                     │
│   - Holds STRIKE stakes     - ADA rewards pool      │
│   - Swap/DEX orders         - Manages ADA distrib.  │
│   - consume_rewards op      - Routes STRIKE to v3   │
│   - 344 UTxOs activos       - Multiple 200M ADA UTxOs│
│                                                     │
│   Ref script: b3e3b7ac#0   Ref script: 2a52d3f7#0  │
└─────────────────────────────────────────────────────┘
                        ↓
              Batcher address: addr1q8x4rlq...
              (procesa txs, cobra fee en STRIKE+ADA)
```

El protocolo usa un **modelo de batcher**:
- Usuarios no interactúan directamente con el script
- Un batcher recoge requests y los procesa en batch
- El batcher cobra fee en STRIKE (ej: 147,773 STRIKE por withdraw)

---

## Estructura real del datum (desplegado)

El datum actual en cadena NO coincide con `StakingDatum` del GitHub público.

### GitHub público (`types.ak`):
```
type StakingDatum {
  owner_address_hash: Hash<Blake2b_224, VerificationKey>,  // 28 bytes
  staked_at: Int,                                          // POSIX ms
  mint_policy_id: PolicyId,                               // 28 bytes
}
```

### Datum real on-chain (CBOR decodificado):
```json
{
  "constructor": 0,
  "fields": [
    {
      // Field 0: Address completa del owner (no solo hash)
      "constructor": 0,
      "fields": [
        {"constructor": 1, "fields": [{"bytes": "<owner_pkh_28bytes>"}]},
        {"constructor": 0, "fields": [{"constructor": 0, "fields": [{"constructor": 0, "fields": [{"bytes": "<stake_pkh>"}]}]}]}
      ]
    },
    {"bytes": "f13ac4d66b3ee19a6aa0f2a22298737bd907cc95121662fc971b5275"},  // staking_policy_id
    {"bytes": "535452494b45"},  // staking_asset_name "STRIKE"
    {"int": 327868852},         // staked_amount
    {"bytes": ""},              // campo desconocido 1
    {"bytes": ""},              // campo desconocido 2
    {"int": 930177177},         // staked_at / last_rewarded timestamp
    {"constructor": 1, "fields": []},  // None
    {"constructor": 0, "fields": [     // tracking rewards
      {"constructor": 0, "fields": [{"bytes": "00"}]},
      {"int": 0}
    ]}
  ]
}
```

---

## Discrepancia tx3 vs contrato desplegado

| Aspecto | tx3 actual (GitHub público) | Contrato desplegado |
|---------|----------------------------|---------------------|
| Credential NFTs | Sí (mint/burn en stake/withdraw) | **NO** — eliminados |
| `stake` tx | Ejecuta Plutus (ref script) | Solo UTxO lock, sin Plutus |
| `owner_address_hash` | `Bytes` (28 bytes PKH) | `Address` completa |
| `mint_policy_id` en datum | 3er campo del datum | No aparece |
| Asset class STRIKE | Solo en `env {}` | Embebido en datum |
| Cantidad stakeada | Solo en outputs | Registrada en datum |
| Redeemer add_stake | `AddStakeOrConsumeStakingRewards {}` | `Constr(1,[Constr(2,[])])` |
| Redeemer withdraw | `WithdrawStake {}` | `Constr(1,[Constr(5,[])])` (a confirmar) |
| `spend_script_ref` | Pendiente | v2: `2a52d3f7...#0`, v3: `b3e3b7ac...#0` |
| `mint_script_ref` | Sí (NFTs) | **No aplica** |
| Batcher | No modelado | Sí, cobra fee en STRIKE |

---

## Datum de V2 vs V3 — diferencias clave

Los UTxOs en V2 con STRIKE real (>1000) tienen un datum radicalmente distinto al de V3:

### Datum en V3 (posición de staking):
- 9 campos: Address completa del owner, asset class STRIKE, staked_amount, timestamps, etc.
- El campo `owner.payment` = `ScriptCredential(V2_HASH)` → **V2 es custodio de V3**
- El campo `owner.staking` = `PubKeyCredential(5ea48152...)` → staking cred del protocolo

### Datum en V2 (proxy/receipt):
```json
{
  "constructor": 0,
  "fields": [{
    "constructor": 0,
    "fields": [
      {"constructor": 0, "fields": [{"bytes": "<tx_hash_32bytes>"}]},
      {"int": <output_index>}
    ]
  }]
}
```
El datum V2 es simplemente una **UTxO reference** que apunta a la posición real en V3. V2 actúa como capa de proxy/custodio.

### Implicación arquitectónica:
- **V3** = almacén real de stakes (owner en datum = V2)
- **V2** = proxy/receipt que referencia el UTxO en V3
- **Batcher** = gestiona el flujo entre V2 y V3

---

## Parámetros compilados del contrato (decodificados del script CBOR)

### v2 — Parámetros:
```
Param 0: Address PKH(0e0b0ac4...) + stake(c85decf1...)  → Batcher address 1
Param 1: Address PKH(f961b231...) + stake(2025a198...)  → Batcher address 2
Param 2: Address Script(1af84a9e...) + stake(5ea48152...)  → v3 contract address
```

### v3 — Parámetros:
```
Param 0: Address PKH(cd51fc17...) + stake(63c28615...)  → Fee address
Param 1: Address PKH(7c2328db...) + stake(8183f129...)  → Authorize address
```

---

## Recomendaciones para el tx3

### Opción A: Mantener basado en GitHub público (actual `main.tx3`)
- Más limpio, posiblemente la versión futura canónica
- No funciona con mainnet actual
- Apropiado si Strike Finance planea redeploy con el código del GitHub

### Opción B: Reescribir para el contrato desplegado
Cambios necesarios:
1. **Eliminar** todo el bloque `mint`/`burn` de las 3 txs
2. **Eliminar** `mint_script_ref` del `env {}`
3. **Eliminar** `tracker_asset_name` del `env {}`
4. **Cambiar** `StakingDatum` a 9 campos con `Address` completa
5. **Cambiar** redeemers a la estructura real desplegada
6. **Simplificar** tx `stake` — no necesita `reference spend_script` ni `collateral`
7. **Agregar** batcher fee output en `add_stake` y `withdraw_stake`
8. **Actualizar** `spend_script_ref` en env:
   - v2: `spend_script_ref = 2a52d3f7be80f0e163a2fbd4fa36703e03a0d5a8139a3828e3156c335be59211#0`
   - v3: `spend_script_ref = b3e3b7acef46f70ef511e7ea91c231a6e090a8a9790837a2b83396cf499203b3#0`

---

## Pendientes

- [ ] Confirmar qué redeemer usa exactamente `withdraw_stake` en v2 (inner=5 probable)
- [ ] Entender la relación exacta entre v2 y v3 (¿cuál es el staking principal?)
- [ ] Confirmar si las txs de Mar 2026 (`add_stake`/`withdraw_stake`) usan v2 o v3
- [ ] Decodificar el datum completo de un UTxO activo en v3 para confirmar la estructura de 9 campos
- [ ] Entender el rol exacto del batcher address (`addr1q8x4...`)
