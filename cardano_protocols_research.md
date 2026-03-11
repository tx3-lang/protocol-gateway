# Investigación de Protocolos Cardano para Transaction Builder

**Fecha:** Marzo 2026  
**Objetivo:** Evaluar la disponibilidad de documentación, código open source, archivos plutus/blueprint, y ejemplos de integración con transaction builders (MeshJS, Lucid, u otros) para los siguientes protocolos:

---

## Resumen Ejecutivo

| Protocolo | Open Source | Plutus/Blueprint | Docs Dev | SDK/Transaction Builder | Dificultad de Integración |
|-----------|-------------|-------------------|----------|-------------------------|---------------------------|
| Fluid (Aquarium) | ✅ Parcial | ⚠️ No confirmado | ✅ Sí | ❌ No oficial | Alta |
| Strike Finance | ✅ Sí (Aiken) | ✅ Sí (blueprint) | ✅ Sí | ⚠️ SDK propio | Media |
| Bodega Market | ✅ Sí | ⚠️ Posible | ✅ Sí | ❌ No oficial | Alta |
| Indigo Protocol | ✅ Sí (BUSL-1.1) | ⚠️ No publicado | ✅ Sí | ⚠️ SDK propio | Alta |
| VyFi | ⚠️ Parcial | ⚠️ Registro parcial | ⚠️ Limitada | ❌ No oficial | Muy Alta |
| Moneta (USDM) | ❌ No | ❌ No | ⚠️ Mínima | ❌ No aplica | N/A (Nativo) |

---

## 1. Fluid Tokens — Protocolo Aquarium

### ¿Qué es?
Aquarium es el protocolo de FluidTokens que permite pagar fees de transacción con tokens nativos en lugar de ADA. Es el primer mercado de fees descentralizado en Cardano. También habilita **transacciones automáticas** (Scheduled Transactions) mediante una red de validadores que ejecutan transacciones cuando se cumplen condiciones predefinidas.

### Open Source
- **Aquarium Node (Java):** [github.com/FluidTokens/ft-aquarium-node](https://github.com/FluidTokens/ft-aquarium-node)
  - Implementado en Java con Yaci Store
  - Indexa UTxOs de los contratos de Aquarium
  - Procesa Scheduled Transactions cuando se cumplen condiciones
- **MVP de Smart Contracts:** [github.com/FluidTokens/ft-aquarium-automatic-sc-mvp](https://github.com/FluidTokens/ft-aquarium-automatic-sc-mvp)
  - Versión MVP de las reglas de Aquarium
- La organización FluidTokens tiene **16 repositorios** en GitHub

### Documentación
- **Docs oficiales:** [docs.fluidtokens.com/cardano/aquarium/](https://docs.fluidtokens.com/cardano/aquarium/)
- **Developer Portal:** [github.com/FluidTokens/developer-portal](https://github.com/FluidTokens/developer-portal)

### Archivos Plutus / Blueprint
- Los smart contracts están listados como open source según la documentación oficial
- **No se encontró un archivo `plutus.json` o blueprint publicado directamente** en los repos encontrados
- Los contratos del nodo usan Blockfrost para interactuar con la cadena

### Integración con Transaction Builder
- **No existe integración oficial con MeshJS o Lucid**
- Para integrarse, se debe:
  1. Consultar el endpoint de Aquarium para obtener los Tanks disponibles
  2. Construir una transacción que incluya los Tanks necesarios (el smart contract verifica que se envíen los tokens requeridos)
  3. Usar MeshJS o Lucid para construir dicha transacción

### Notas Técnicas
```
Componentes principales:
- FeeTanks: UTxOs con ADA para patrocinar fees
- Aquarium Lambdas: condiciones bajo las cuales se puede gastar ADA de los tanks
- Validadores: red de operadores con 30k FLDT en stake
- Parámetros: ratio ADA/token (estático o dinámico por oracle)
```

### Recursos
- Docs: https://docs.fluidtokens.com/protocols/aquarium/
- GitHub Org: https://github.com/fluidtokens

---

## 2. Strike Finance

### ¿Qué es?
Protocolo de derivados descentralizado en Cardano. Ofrece trading de **opciones, forwards y contratos perpetuos** (futuros sin fecha de vencimiento). Strike V1 se lanzó en mainnet en mayo 2025 utilizando el modelo GMX original, donde los proveedores de liquidez actúan como contraparte de los traders.

### Open Source
Strike Finance es **completamente open source**. Sus repositorios en GitHub:
- **Perpetuals:** [github.com/strike-finance/perpetuals-smart-contracts](https://github.com/strike-finance/perpetuals-smart-contracts)
- **Forwards:** [github.com/strike-finance/forwards-smart-contracts](https://github.com/strike-finance/forwards-smart-contracts)
- **Options:** [github.com/strike-finance/options-smart-contracts](https://github.com/strike-finance/options-smart-contracts)
- **Staking:** [github.com/strike-finance/staking-smart-contracts](https://github.com/strike-finance/staking-smart-contracts)
- **SDK v1:** [github.com/strike-finance/strike-sdk-v1](https://github.com/strike-finance/strike-sdk-v1)

### Lenguaje de Smart Contracts
Los contratos están escritos en **Aiken** (lenguaje moderno de Cardano que compila a Plutus Core). Requiere Aiken instalado en el PATH:
```bash
# Agregar Aiken al PATH en ~/.zshrc o ~/.bashrc
```

### Archivos Plutus / Blueprint
- Al usar **Aiken**, se genera automáticamente un archivo `plutus.json` (blueprint) al compilar
- El blueprint contiene las definiciones de validadores, tipos de datum/redeemer y hashes de los scripts
- Los repos de Aiken típicamente incluyen `plutus.json` en la raíz o en el directorio `build/`

### SDK / Transaction Builder
- **Strike SDK v1** disponible en GitHub
- Permite interactuar con los contratos desde JavaScript/TypeScript
- Tiene integración con **hummingbot** para bots de trading

### Auditoría
- Auditoría de los contratos perpetuos realizada y publicada en el repo

### Documentación
- **Docs oficiales:** [docs.strikefinance.org](https://docs.strikefinance.org/)
- Tiene documentación detallada de los contratos

### Notas Técnicas
```
Modelo de perpetuos (GMX-style):
- Traders abren posiciones long/short con apalancamiento
- Collateral requerido (puede ser el activo subyacente o stablecoin)
- Liquidación cuando el collateral cae a un % de su valor original
- STRIKE como colateral adicional (quema en liquidación)
- Stop loss / take profit automatizados
- Funding rate para mantener precio perpetuo alineado con spot
```

---

## 3. Bodega Market

### ¿Qué es?
Plataforma de **mercados de predicción descentralizados** sobre la blockchain de Cardano. Permite a usuarios crear, tradear y resolver mercados de predicción sobre eventos del mundo real (deportes, política) y eventos on-chain (precio de ADA, MIN, SNEK, etc.).

### Open Source
- **Smart Contracts V2:** [github.com/bodega-market/bodega-market-smart-contracts-v2](https://github.com/bodega-market/bodega-market-smart-contracts-v2)
- **Smart Contracts V1:** [github.com/bodega-market/bodega-market-smart-contracts](https://github.com/bodega-market/bodega-market-smart-contracts)
  - Nota: V1 todavía en alpha, no recomendado para producción
- **Docs:** [github.com/bodega-market/bodega-market-docs](https://github.com/bodega-market/bodega-market-docs)

### Archivos Plutus / Blueprint
- El contrato V2 está documentado con las instrucciones de deployment
- **No se encontró un archivo blueprint público explícito** en las búsquedas
- El V1 está en alpha y sujeto a cambios breaking

### Flujo del Protocolo (según V2)
```
1. Configuración del protocolo:
   - Mint de tokens de settings y autenticación del manager
   - Envío a script addresses correspondientes
   - Mint de reference script tokens

2. Configuración del proyecto:
   - Mint de authentication tokens
   - Envío a project info UTxO con pledge
   - Transferencia de open fee al treasury

3. Participación del usuario:
   - Usuario envía payment tokens a script address con datum
   - Batcher colecta posiciones y las aplica al proyecto
   - Usuario recibe share tokens proporcionales

4. Resolución:
   - Distribución de rewards entre creador del proyecto y protocolo
   - Ratio determinado por share_ratio en project info datum
```

### Documentación
- **Docs oficiales:** [docs.bodegacardano.org](https://docs.bodegacardano.org)
- Tiene sección de protocolo con contratos, características, staking, etc.

### Integración con Transaction Builder
- **No existe integración oficial con MeshJS o Lucid**
- Se puede implementar usando los contratos V2 como referencia

---

## 4. Indigo Protocol

### ¿Qué es?
Protocolo de **activos sintéticos** autónomo en Cardano. Permite crear iAssets (activos sintéticos) que replican el precio de activos del mundo real (iBTC, iETH, iUSD, etc.) usando CDPs (Collateral Debt Positions) con ADA o stablecoins como colateral.

### Open Source
Indigo **abrió su código fuente en abril 2023**:
- **Smart Contracts V1:** [github.com/IndigoProtocol/indigo-smart-contracts](https://github.com/IndigoProtocol/indigo-smart-contracts)
  - **Licencia: Business Source License 1.1 (BUSL-1.1)** — No es libre para uso comercial hasta que expire
- **SDK:** [github.com/IndigoProtocol/indigo-sdk](https://github.com/IndigoProtocol/indigo-sdk)

### Evolución del Lenguaje
- **V1:** Escrito en **PlutusTx** con Plutonomy optimizer
- **V2 (en desarrollo):** Migración a **Aiken** para mejor eficiencia (menor uso de CPU/memoria)
  - Pruebas mostraron 40-60% de reducción en execution units

### Archivos Plutus / Blueprint
- Los contratos V1 compilados están disponibles en el repositorio
- El repo incluye benchmarks en YAML con límites de execution units
- **Para V2 (Aiken):** Generará blueprint automáticamente al compilar

### SDK
- `indigo-sdk` disponible en GitHub
- Permite calcular rewards INDY y otras interacciones

### Documentación
- **Docs oficiales:** [docs.indigoprotocol.io](https://docs.indigoprotocol.io) (inferido)
- Bug bounty program activo desde abril 2023

### Notas Técnicas
```
Componentes del protocolo:
- CDPs: Minting de iAssets colateralizando ADA (mín 200%) o stablecoins (mín 150%)
- Stability Pools: Liquidación de CDPs insolventes; stabilitiy providers reciben colateral
- Governance (INDY): Votación de parámetros del protocolo
- Liquid Staking: ADA en CDP sigue generando staking rewards
- Oráculos: Precio de iAssets feed por oráculos descentralizados
```

### Restricción BUSL
> ⚠️ **Importante:** La licencia BUSL-1.1 prohíbe uso comercial de los contratos V1 hasta una fecha de conversión. Para integración en productos comerciales, verificar términos de licencia o usar V2 (Aiken) cuando esté disponible.

---

## 5. VyFi (VyFinance)

### ¿Qué es?
Protocolo DeFi en Cardano con múltiples productos: **DEX (AMM)**, BAR (mecanismo redistributivo), governance, lotería, y token/NFT Vaults. También tiene un Auto-Harvester que gestiona yield farming usando una Neural Net.

### Open Source
La organización VYFI en GitHub tiene repositorios **limitados en scope público**:
- **Cardano Contracts Registry:** [github.com/VYFI/cardano-contracts-registry](https://github.com/VYFI/cardano-contracts-registry)
  - Contiene registro de contratos Cardano
- **Metadata Registry Testnet:** [github.com/VYFI/metadata-registry-testnet](https://github.com/VYFI/metadata-registry-testnet)

### Archivos Plutus / Blueprint
- **No se encontraron archivos plutus.json o blueprints publicados**
- El `cardano-contracts-registry` puede contener direcciones de scripts pero no el código fuente compilado
- Los contratos principales del DEX **no parecen estar open source públicamente**

### Documentación
- **Docs oficiales:** [docs.vyfi.io](https://docs.vyfi.io)
- Documentación de usuario disponible pero **sin documentación técnica de contratos para developers**

### Integración con Transaction Builder
- **No existe integración oficial con MeshJS, Lucid u otros**
- Sin acceso a los contratos compilados, la integración requeriría:
  1. Reverse-engineering de transacciones existentes
  2. Contactar directamente al equipo de VyFi
  3. Usar su API (si existe)

### Estado
> ⚠️ **VyFi es el protocolo con menor disponibilidad de recursos para developers de los 6 investigados.** No se encontró código de contratos público, blueprints, ni SDK. La integración directa en un transaction builder sería significativamente más compleja que los demás protocolos.

---

## 6. Moneta (USDM)

### ¿Qué es?
**USDM** es la stablecoin fiat-backed principal de Cardano, emitida por Moneta Digital LLC (empresa registrada como Money Services Business ante FinCEN en EE.UU.). Cada USDM está respaldado 1:1 por dólares en reservas (Fidelity + Western Asset Management). También tiene co-emisor en Europa: **NBX** (bajo regulación MiCA).

### Naturaleza del Token
USDM es un **Cardano Native Token** — **no es un smart contract**. Es un token nativo creado con una minting policy controlada por Moneta Digital. Esto es fundamentalmente diferente a los otros protocolos:

- No requiere smart contracts para transferir
- Las transacciones son simples transfers de native token
- Solo Moneta (y NBX en EU) pueden mintear/quemar USDM

### Open Source
- **No hay código open source del protocolo de minting**
- El proceso de mint/burn es centralizado y controlado por Moneta

### Archivos Plutus
- La minting policy puede ser un script simple o multi-sig
- **No está publicado el script de minting policy**
- Se puede obtener el Policy ID del token consultando on-chain:
  - Policy ID de USDM: identificable en exploradores como cexplorer.io o pool.pm

### Integración con Transaction Builder
Como native token, USDM se puede usar en transacciones normales de Cardano:
```typescript
// Ejemplo con MeshJS para enviar USDM
import { MeshTxBuilder } from "@meshsdk/core";

const tx = new MeshTxBuilder({ fetcher: provider });
await tx
  .txOut(recipientAddress, [
    { unit: "USDM_POLICY_ID" + "USDM", quantity: "1000000" } // 1 USDM (6 decimales)
  ])
  .complete();
```

### Para minting/redención
- Requiere KYC y cuenta en Moneta (moneta.global) o NBX
- Monto mínimo de mint: $1,000 USD
- El proceso es off-chain (depósito bancario → mint on-chain)

### Policy ID de USDM
```
Para obtener el Policy ID actual de USDM, consultar:
- https://cardanoscan.io → buscar "USDM"
- O consultar directamente en la documentación de Moneta
```

---

## Recomendaciones para Transaction Builder

### Protocolos con mayor viabilidad de integración

#### 🟢 Alta Viabilidad: Strike Finance
- Contratos en Aiken → blueprint generado automáticamente
- SDK propio disponible
- Documentación de contratos buena
- Open source completo

#### 🟡 Media Viabilidad: Bodega Market & Indigo Protocol
- Código fuente disponible pero sin SDKs para builders externos
- Requiere leer contratos y construir integración manual
- Indigo tiene restricción BUSL-1.1

#### 🟡 Media Viabilidad: Fluid Tokens (Aquarium)
- La lógica de "fee sponsoring" es única y valiosa para UX
- Hay un developer portal con API endpoints
- La integración requiere consultar la API de Aquarium + construir tx

#### 🔴 Baja Viabilidad: VyFi
- Sin contratos públicos disponibles
- Sin SDKs oficiales
- Contactar directamente al equipo

#### ⚪ N/A: Moneta (USDM)
- Es un native token, se integra nativamente en cualquier transaction builder
- No requiere lógica de smart contract para usar USDM como token de pago

---

## Recursos Adicionales de Transaction Builders para Cardano

| Tool | Lenguaje | Link |
|------|----------|------|
| **MeshJS** | TypeScript | meshjs.dev |
| **Lucid** (deprecated → Lucid Evolution) | TypeScript | github.com/lucid-evolution |
| **PyCardano** | Python | pycardano.readthedocs.io |
| **cardano-serialization-lib** | Rust/WASM | github.com/Emurgo/cardano-serialization-lib |
| **Aiken** | Aiken/Plutus | aiken-lang.org |
| **Atlas** | Haskell | github.com/geniusyield/atlas |

---

## Análisis de Viabilidad con Tx3 (Ingeniería Inversa)

### ¿Qué es Tx3?

**Tx3** es un DSL (Domain Specific Language) creado por [TxPipe](https://txpipe.io/) para describir la interfaz de protocolos UTxO en Cardano. Es el "OpenAPI" del mundo eUTxO: permite definir templates de transacciones como funciones parametrizadas, y luego genera bindings de código en TypeScript, Rust, Go o Python.

> Tx3 **no reemplaza los contratos on-chain** (eso sigue siendo Aiken/PlutusTx). Solo describe cómo interactuar con ellos off-chain de forma declarativa.

**Componentes clave del lenguaje:**
- `party` — participantes de la transacción (wallet o script)
- `policy` — script on-chain (validador o minting policy)
- `record` — estructuras de datos para datums y redeemers
- `tx` — template de transacción con inputs, outputs y lógica

**Tooling:**
- `trix` — CLI para init, build, test y bindgen de proyectos Tx3
- `tx3up` — instalador del ecosistema
- VSCode extension — syntax highlighting, diagramas, formulario de testing
- Devnet — red local de pruebas integrada

### Proceso General de Ingeniería Inversa para Tx3

Para los protocolos sin blueprint público, el flujo de trabajo sería:

1. **Identificar script addresses** on-chain (desde UI del protocolo o documentación)
2. **Explorar transacciones históricas** en CardanoScan o Cexplorer
3. **Decodificar datums CBOR** → inferir tipos y campos
4. **Identificar redeemers** → qué acciones acepta cada validador
5. **Escribir los `record`** en Tx3 con los tipos inferidos
6. **Escribir los `tx` templates** replicando los patrones de UTxOs observados
7. **Referenciar el script** con `policy NombreScript = import(script.plutus)` o como dirección fija

### Viabilidad por Protocolo con Tx3

#### 🟢 Strike Finance — Alta viabilidad

**Esfuerzo estimado: 1-2 días**

Es el caso ideal para Tx3. Los contratos están escritos en Aiken, que genera automáticamente un `plutus.json` con tipos de datum y redeemer completamente tipados. Con ese blueprint se pueden mapear directamente los `record` y `tx` en Tx3 sin ingeniería inversa real.

```
// Ejemplo conceptual de cómo quedaría el .tx3
policy Perpetuals = import(build/perpetuals.plutus);

record OpenPositionDatum {
  trader: Bytes,
  collateral: Int,
  direction: Int,    // 0 = long, 1 = short
  leverage: Int,
}

tx openPosition(quantity: Int, leverage: Int) {
  input source {
    from: Trader,
    min_amount: quantity,
  }
  output position {
    to: Perpetuals,
    amount: Ada(quantity),
    datum: OpenPositionDatum { ... }
  }
}
```

**Ventajas:**
- Blueprint con tipos completos disponible
- SDK propio como referencia para validar implementación
- Auditoría publicada facilita entender los flujos

---

#### 🟡 Indigo Protocol — Viabilidad media

**Esfuerzo estimado: 3-5 días**

El código fuente en PlutusTx (Haskell) está publicado, lo que permite leer los tipos de datum y redeemer directamente. La arquitectura es compleja (CDPs, Stability Pools, oráculos, governance) pero bien documentada en el repositorio. El mayor obstáculo es la **licencia BUSL-1.1** que restringe uso comercial.

Para V2 (migración a Aiken, en desarrollo), la viabilidad aumenta significativamente ya que habrá blueprint automático.

**Bloqueantes:**
- Licencia BUSL-1.1: verificar términos antes de integraciones comerciales
- Múltiples validadores interrelacionados complican los templates
- Oráculos descentralizados requieren lógica adicional de referencing

---

#### 🟡 Bodega Market — Viabilidad media

**Esfuerzo estimado: 3-5 días**

Los contratos V2 están en GitHub. Sin embargo, la documentación técnica interna (tipos de datum, estructura de redeemers) no está detallada en los docs. Requiere leer el código fuente para derivar las estructuras. El modelo de **batcher** (que agrega posiciones de usuarios antes de aplicarlas al contrato) introduce un patrón UTxO más complejo de describir en Tx3.

**Bloqueantes:**
- Flujo de batcher no es trivial de modelar en templates simples
- Sin blueprint generado; los tipos hay que inferirlos del código
- V1 en alpha no recomendado; V2 es la versión a usar

---

#### 🟠 Fluid Tokens (Aquarium) — Viabilidad media-baja

**Esfuerzo estimado: 5-8 días**

La lógica de FeeTanks tiene flujos atípicos: el contrato verifica ratios ADA/token dinámicos vía oráculos, y el flujo de "sponsorear fees" implica UTxOs de múltiples partes coordinadas. Sin blueprint público, habría que analizar transacciones reales on-chain para decodificar la estructura de datums. El developer portal con endpoints de API ayuda a entender los parámetros disponibles, pero no reemplaza el conocimiento del contrato.

**Bloqueantes:**
- Sin blueprint ni código de contratos publicado explícitamente
- Ratios dinámicos via oracle requieren referencing de UTxOs externos
- El modelo de múltiples Tanks por transacción es complejo de parametrizar en Tx3

---

#### 🔴 VyFi — Baja viabilidad

**Esfuerzo estimado: 2-3 semanas o más**

Es el caso de ingeniería inversa "pura". Sin contratos publicados, sin blueprint, sin SDK y sin documentación técnica, la única opción es analizar exhaustivamente transacciones históricas en exploradores, decodificar cada datum CBOR manualmente, e inferir la estructura completa del protocolo. Es factible en teoría, pero el costo en tiempo es desproporcionado respecto a los otros protocolos.

**Alternativa recomendada:** Contactar directamente al equipo de VyFi para solicitar documentación técnica o acceso a los contratos antes de intentar la ingeniería inversa.

---

### Tabla Resumen de Viabilidad Tx3

| Protocolo | Viabilidad Tx3 | Fuente de Tipos | Esfuerzo estimado | Bloqueante principal |
|-----------|---------------|-----------------|-------------------|----------------------|
| Strike Finance | 🟢 Alta | Blueprint Aiken (auto-generado) | 1-2 días | Ninguno relevante |
| Indigo Protocol | 🟡 Media | Código PlutusTx en GitHub | 3-5 días | Licencia BUSL-1.1 |
| Bodega Market | 🟡 Media | Código fuente V2 en GitHub | 3-5 días | Modelo de batcher |
| Fluid Tokens | 🟠 Media-baja | On-chain CBOR + API endpoints | 5-8 días | Sin blueprint público |
| VyFi | 🔴 Baja | On-chain CBOR únicamente | 2-3 semanas+ | Sin código fuente |
| Moneta (USDM) | ⚪ N/A | Native token (no aplica) | Horas | Ninguno |

### Orden Recomendado de Implementación

1. **Strike Finance** — Para aprender el flujo de Tx3 con un caso limpio y bien documentado
2. **Indigo Protocol** — Si la licencia BUSL-1.1 no es un bloqueante; código fuente claro
3. **Bodega Market** — Código disponible; requiere entender el modelo de batcher
4. **Fluid Tokens** — Útil para UX (fee sponsoring), pero requiere más análisis on-chain
5. **VyFi** — Solo si los otros están completos o si el equipo provee documentación técnica

---

*Investigación realizada en marzo 2026. Los proyectos DeFi de Cardano evolucionan rápidamente; verificar repositorios directamente para información actualizada.*
