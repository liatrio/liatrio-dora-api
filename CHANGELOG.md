# CHANGELOG

## v0.3.2 (2024-07-18)

### Fix

* fix: add more logging output ([`4ea9590`](https://github.com/liatrio/liatrio-dora-api/commit/4ea9590dce4b579540dc0639aa444ae11c5351ac))

## v0.3.1 (2024-07-17)

### Fix

* fix: Allow no auth (#14)

* fix: allow for no auth when talking to loki

* fix: if an error occurs, log it ([`dedc733`](https://github.com/liatrio/liatrio-dora-api/commit/dedc7331896c5db37cf6839902f9101658995dde))

## v0.3.0 (2024-07-16)

### Feature

* feat: add caching to the data endpoint ([`d8062d2`](https://github.com/liatrio/liatrio-dora-api/commit/d8062d21048b5977e6265642c6fed112bf26732c))

## v0.2.9 (2024-07-11)

### Fix

* fix: issue failed/fixed dates were incorrect, query was broken ([`5f14361`](https://github.com/liatrio/liatrio-dora-api/commit/5f14361cd7dec68b0a7772c206ca9ab18bd9541c))

## v0.2.8 (2024-07-10)

### Fix

* fix: force version bump ([`4304ff4`](https://github.com/liatrio/liatrio-dora-api/commit/4304ff44e01a8629dfdf6e9fb9e0c6bdcce5bd69))

### Unknown

* Adjust failed flag (#13)

* fix: Adjust failed flag to be separate from status

* fix: include issue closed dates

* fix: Make dates optional instead of defaulting to min, fix fail tracking ([`e8650e6`](https://github.com/liatrio/liatrio-dora-api/commit/e8650e64ac8fdc72da21767dd86e92e56abeba90))

* Update README.md ([`3676b2d`](https://github.com/liatrio/liatrio-dora-api/commit/3676b2d271bc149e9bc0fcac781297ae33bd4f1d))

## v0.2.7 (2024-07-03)

### Fix

* fix: use correct data for final send ([`85ce4a7`](https://github.com/liatrio/liatrio-dora-api/commit/85ce4a7f8b98105e458b7e4ea84f3d45f28f510d))

## v0.2.6 (2024-07-03)

### Fix

* fix: fixed_at was being applied to wrong recordd ([`169a6c7`](https://github.com/liatrio/liatrio-dora-api/commit/169a6c7e6cce668b2c84d6be5535430f8bf0eb9c))

## v0.2.5 (2024-07-02)

### Fix

* fix: focus on dev fttb, don&#39;t rely on pr open ([`37311e9`](https://github.com/liatrio/liatrio-dora-api/commit/37311e970a29a1d1c4a1822eb6ecf87e3ffc8a45))

## v0.2.4 (2024-07-02)

### Fix

* fix: update issue query to go off opened rather than labeled ([`84a2b4a`](https://github.com/liatrio/liatrio-dora-api/commit/84a2b4a85a3f97f3f40f746434be70ecf94e88e0))

## v0.2.3 (2024-07-01)

### Fix

* fix: update df to use same naming as everything else ([`036a144`](https://github.com/liatrio/liatrio-dora-api/commit/036a1448d45afbb6d62aa443bbef44d40f4acb78))

## v0.2.2 (2024-06-20)

### Fix

* fix: update readme ([`f21a2ea`](https://github.com/liatrio/liatrio-dora-api/commit/f21a2eaa61cf2a249df0db2a604836d4d428f328))

### Unknown

* Condense API Endpoints (#11)

* feat: implementing CFR

* fix: adjust for missing repo name

* fix: don&#39;t include state, it&#39;s not necessary

* implementing RT

* fix: add unified data endpoint that supplies all data from the other 4

---------

Co-authored-by: Eliot Eikenberry &lt;wolftousen@Tundria.local&gt; ([`388ff58`](https://github.com/liatrio/liatrio-dora-api/commit/388ff58d60da1eecbc08bae8375ba68f7dd43e4e))

* Implementing CFR and RT (#9)

* feat: implementing CFR

* fix: adjust for missing repo name

* fix: don&#39;t include state, it&#39;s not necessary

* implementing RT

---------

Co-authored-by: Eliot Eikenberry &lt;wolftousen@Tundria.local&gt; ([`f3553be`](https://github.com/liatrio/liatrio-dora-api/commit/f3553bed2f5a8ab8f24b0297d2c187efdd5322c0))

* Add catalog-info.yaml config file (#8) ([`4fd3a9b`](https://github.com/liatrio/liatrio-dora-api/commit/4fd3a9bb1349bcbbe61389bcd603330d259db24c))

## v0.2.1 (2024-06-04)

### Fix

* fix: add readme, update cfr and rt skeletons ([`e578085`](https://github.com/liatrio/liatrio-dora-api/commit/e578085aab920b747868c09797ea5fd3a17058a4))

## v0.2.0 (2024-05-31)

### Feature

* feat: implemenet CLT and refactor ([`52c95c1`](https://github.com/liatrio/liatrio-dora-api/commit/52c95c1be0df80876db516219e95cbfdc67ab30a))

## v0.1.7 (2024-05-31)

### Fix

* fix: swap to ipv6 ([`56fc383`](https://github.com/liatrio/liatrio-dora-api/commit/56fc3838fccce4758570bf1c5140cb1617fc4d79))

## v0.1.6 (2024-05-29)

### Chore

* chore: fix warnings ([`84ed448`](https://github.com/liatrio/liatrio-dora-api/commit/84ed448ab2579afb4047cf49af5b8d84eeac5f5a))

### Fix

* fix(addr): Use 0.0.0.0 instead of 127.0.0.1 ([`e80b283`](https://github.com/liatrio/liatrio-dora-api/commit/e80b28345a3c964839e94c0e3399b4401f5c9e25))

### Unknown

* feature(df): implement df loki query ([`6ac2828`](https://github.com/liatrio/liatrio-dora-api/commit/6ac2828bccb96d93125a8ec3c4e6122f4332eafc))

## v0.1.5 (2024-05-29)

### Fix

* fix(Docker): invalid entry point ([`7d65e0b`](https://github.com/liatrio/liatrio-dora-api/commit/7d65e0b0bd67b46bcfc324496b05fe5884e4a0c8))

## v0.1.4 (2024-05-24)

### Fix

* fix(workflow): prevent looped workflow triggers of the release pipeline ([`c860a04`](https://github.com/liatrio/liatrio-dora-api/commit/c860a044f7722c9c97929479122787d7c0055c10))

## v0.1.3 (2024-05-24)

### Fix

* fix(workflow): use the embedded gitref for tagging ([`32ab524`](https://github.com/liatrio/liatrio-dora-api/commit/32ab524914a9bfc3e142b2dae50878604c3a8659))

* fix(workflow): prevent workflow loop on tag push ([`6ae0a2c`](https://github.com/liatrio/liatrio-dora-api/commit/6ae0a2c6d265706cdd8a6c24314818db1d7b7f54))

## v0.1.2 (2024-05-24)

### Fix

* fix(token): use a PAT token so the builds can trigger properly ([`35abbe9`](https://github.com/liatrio/liatrio-dora-api/commit/35abbe9c3239d19b84b615239a0209ef093196d9))

## v0.1.1 (2024-05-24)

### Fix

* fix(manifest): use correct secret ref name ([`851133e`](https://github.com/liatrio/liatrio-dora-api/commit/851133e56f79d829181923ace3e190af6236dd1c))

## v0.1.0 (2024-05-24)

### Feature

* feat(n/a): force minor version bump ([`b598608`](https://github.com/liatrio/liatrio-dora-api/commit/b5986088aa34d4a1db2caa659efa4b9f677defb2))

## v0.0.0 (2024-05-24)

### Unknown

* force build&#39; ([`052b73f`](https://github.com/liatrio/liatrio-dora-api/commit/052b73f8ba289479fc03788a07df29558cb1a410))

* fetch tags ([`f03cb51`](https://github.com/liatrio/liatrio-dora-api/commit/f03cb51d14090d95fd82493261b1928b23ecd2bb))

* fetch tags ([`21fadae`](https://github.com/liatrio/liatrio-dora-api/commit/21fadae5fea4753621d641ad98a70947fe502f98))

* find tag a different way ([`52f5ed8`](https://github.com/liatrio/liatrio-dora-api/commit/52f5ed82d705ad320f08fbc054c82b454c1db41f))

* force deploy ([`29157ab`](https://github.com/liatrio/liatrio-dora-api/commit/29157ab1c0a5a8951ae4a7ebb9115ed348c7476d))

* use release tag as docker tag ([`003214b`](https://github.com/liatrio/liatrio-dora-api/commit/003214bbada20588eb8d2d4e507c95fad7982f82))

* fix build workflow error ([`27e203b`](https://github.com/liatrio/liatrio-dora-api/commit/27e203bbc00719ba496a250652e7341e91a75704))

* Update github token ([`db49379`](https://github.com/liatrio/liatrio-dora-api/commit/db493799257dde19d9cd2e942a98da100cd47aeb))

* force release ([`2114935`](https://github.com/liatrio/liatrio-dora-api/commit/21149351b389d58ce968f18e98ce045b13462314))

* force release ([`4439830`](https://github.com/liatrio/liatrio-dora-api/commit/4439830eba332165212b450b004ca7b03b607c23))

* change to build on release ([`322875d`](https://github.com/liatrio/liatrio-dora-api/commit/322875d7217ca84c9638f1ccec11de45f60fc7bf))

* fix typo ([`b0cfd92`](https://github.com/liatrio/liatrio-dora-api/commit/b0cfd92d5dad85fa75d93436ebc7a143f4db4305))

* allow manual running of workflow ([`a8837bb`](https://github.com/liatrio/liatrio-dora-api/commit/a8837bb98f9304a3514b71c95512d1e7a0e1bea4))

* tag docker image with semantic version ([`d065a80`](https://github.com/liatrio/liatrio-dora-api/commit/d065a80db7b0773f01bb89a21a91287a149912e2))

* fixing env var type error ([`30bcf23`](https://github.com/liatrio/liatrio-dora-api/commit/30bcf235d01940b2ebd72560b1c8d67bb8b9a79e))

* fix spacing in yaml file ([`6defa2d`](https://github.com/liatrio/liatrio-dora-api/commit/6defa2dfa7514f0a059f5cbe4e73da2de2ae2697))

* update route name ([`fbeb167`](https://github.com/liatrio/liatrio-dora-api/commit/fbeb1677b28c203a63a2e3a45a6770e89c2e0e86))

* fix folder name ([`95eae6c`](https://github.com/liatrio/liatrio-dora-api/commit/95eae6c93e1b8d546e9144acd31a9596203f9ffd))

* adding manifest files ([`b62b846`](https://github.com/liatrio/liatrio-dora-api/commit/b62b8463bce88d9e0ea91c2b0ad11d2c422fae42))

* add more docker tags ([`b5c2413`](https://github.com/liatrio/liatrio-dora-api/commit/b5c24136d25fc76e2ed93dc7f6b1360b96f341a9))

* add semantic release pipeline ([`c3cc290`](https://github.com/liatrio/liatrio-dora-api/commit/c3cc29011954e13636ff61794f336598c3eb8511))

* add health check endpoint ([`163a005`](https://github.com/liatrio/liatrio-dora-api/commit/163a005bb9a99ffbde7c4c24dac5a93112a6a280))

* remove job dependency ([`7aef22f`](https://github.com/liatrio/liatrio-dora-api/commit/7aef22fa54e08ffc670e198dad5c134cd693e459))

* Adjusting Dockerfile and changing how OpenSSH is referenced ([`77f4d95`](https://github.com/liatrio/liatrio-dora-api/commit/77f4d9552c6f2fc09cfba4cf2e2e1c84e0dcd038))

* remove targeting ([`0723e69`](https://github.com/liatrio/liatrio-dora-api/commit/0723e6914e65b8bb85ce7776481ee485afdcfa7b))

* split these commands ([`560136e`](https://github.com/liatrio/liatrio-dora-api/commit/560136e697a5e3e3255e276504bbf7312585ff4a))

* adjust build dependencies ([`966dd57`](https://github.com/liatrio/liatrio-dora-api/commit/966dd575ee1ff03004e873d0700f5fe3812b9370))

* remove rustls-tls ([`18c42ec`](https://github.com/liatrio/liatrio-dora-api/commit/18c42ec99ba90e0b220cb448c5fdc896f928dc4e))

* use docker to build the app ([`30e3deb`](https://github.com/liatrio/liatrio-dora-api/commit/30e3deb3c0df1e4e68e73015cad95bc11b6d2531))

* swap openssl for rustls-tls ([`6fdd340`](https://github.com/liatrio/liatrio-dora-api/commit/6fdd340bc0cb4e40052dfe1e9cd3a005aba2a06f))

* try one last flag ([`876ef49`](https://github.com/liatrio/liatrio-dora-api/commit/876ef4970bf5678e5ac6e66311f082fa1c7ebb6b))

* fix bad package name ([`73b8d82`](https://github.com/liatrio/liatrio-dora-api/commit/73b8d82e4552587e8eb4e6ecf5938fa93537383a))

* more dependencies ([`1b20185`](https://github.com/liatrio/liatrio-dora-api/commit/1b201850f43f07759c1ea032c8945a4d35962f28))

* add missing dir ([`16b4a30`](https://github.com/liatrio/liatrio-dora-api/commit/16b4a301668aa74d490cc4ecfd39d8f0b99533e4))

* fix paths ([`ece79d8`](https://github.com/liatrio/liatrio-dora-api/commit/ece79d81830e855d3365233c94700a47ca44a2c8))

* make sure target is installed ([`5013bdd`](https://github.com/liatrio/liatrio-dora-api/commit/5013bdddc024edcad4d0ecc145746781a4607b98))

* more build dependencies ([`31fbd08`](https://github.com/liatrio/liatrio-dora-api/commit/31fbd08f7131b35d41c40ea40bae6437f62b4caf))

* another build dependency ([`8ee7901`](https://github.com/liatrio/liatrio-dora-api/commit/8ee790192924b98afe065a1f1ced60b53088f694))

* add missing compilation flag ([`18cf68a`](https://github.com/liatrio/liatrio-dora-api/commit/18cf68a7d0428a56da3842f4b028422c7ed6efb0))

* add missing build dependencies ([`977f8d5`](https://github.com/liatrio/liatrio-dora-api/commit/977f8d588e9ebc8909a6ee838baa80d51bee3e74))

* setup build workflow and docker image ([`1ba2185`](https://github.com/liatrio/liatrio-dora-api/commit/1ba21851037243fcef98de409fd79a6a9332534b))

* initial checkin of basic rust rest api for standard dora endpoints ([`0cbd8c4`](https://github.com/liatrio/liatrio-dora-api/commit/0cbd8c4f0c2dff1d38b5a0d8ddc7c666d29e99cd))

* Initial commit ([`4a4e324`](https://github.com/liatrio/liatrio-dora-api/commit/4a4e3242fd31d02d523d7562f9ea7d22fe9116f5))
