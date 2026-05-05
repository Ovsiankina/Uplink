# Concept de Test — Uplink

---

| | |
|---|---|
| **Projet** | Test complet de l'application Uplink |
| **Application** | Uplink — Messagerie P2P chiffrée |
| **Repository** | github.com/Ovsiankina/Uplink |
| **Version** | 0.1 |
| **Date** | 26.03.2026 |

### Historique des révisions

| Date | Version | Description | Auteur |
|------|---------|-------------|--------|
| 26.03.2026 | 0.1 | Création initiale du concept de test | Mostoslavski David |

---

## Table des matières

1. Objectifs du concept
2. Objets à tester
3. Périmètre de test et priorisation
4. Types de tests
5. Infrastructure de test
6. Organisation des tests
7. Plan de test
8. Classification des défauts
9. Critères d'entrée et de sortie
10. Glossaire

---

## 1. Objectifs du concept

### 1.1 Objectifs primaires

Ce concept de test définit la stratégie, l'organisation et les moyens nécessaires pour valider la qualité de l'application **Uplink**. L'objectif principal est de garantir que les fonctionnalités critiques de l'application fonctionnent correctement dans tous les scénarios d'utilisation prévus, en couvrant les cas nominaux, les cas d'exception et les cas limites.

Les objectifs visés sont :

- Vérifier le bon fonctionnement des fonctionnalités cœur de l'application
- Identifier et documenter les défauts existants avant une mise en production
- Assurer un niveau de confiance suffisant dans la qualité du logiciel
- Établir une base de tests automatisés réutilisable pour les itérations futures

### 1.2 Objectifs secondaires

- Identifier les vulnérabilités potentielles liées à l'injection de contenu malveillant (XSS) dans les messages
- Évaluer la robustesse des fonctions de formatage face à des entrées inhabituelles ou adverses
- Documenter les bonnes pratiques de test pour faciliter la montée en compétences de l'équipe sur le projet

### 1.3 Critères de couverture

L'objectif de couverture de code est fixé à **80% minimum** sur les fonctions pures du périmètre sélectionné (cf. section 3). Les composants UI seront couverts partiellement par les tests d'intégration et E2E.

---

## 2. Objets à tester

### 2.1 Description générale de l'application

**Uplink** est une application de messagerie P2P (pair-à-pair) sécurisée et chiffrée de bout en bout. Elle est construite au-dessus de Warp, IPFS et LibP2P. L'interface utilisateur est développée intégralement en Rust avec le framework Dioxus. L'application est actuellement en phase alpha.

Les principales fonctionnalités d'Uplink sont :

- Messagerie texte chiffrée en temps réel entre pairs
- Envoi et réception de fichiers
- Formatage des messages en Markdown
- Réactions par emojis sur les messages
- Gestion de contacts et demandes d'amis
- Appels audio/vidéo (via WebRTC)
- Système d'extensions modulaire
- Gestion de profil utilisateur (avatar, statut, identifiant décentralisé)

### 2.2 Architecture technique

Le projet est organisé en un workspace Rust composé de plusieurs crates (libraries ou packages) :

| Crate | Rôle |
|---|---|
| `ui` | Application principale, point d'entrée, gestion des pages et de la navigation |
| `kit` | Bibliothèque de composants UI réutilisables (messages, fichiers, contacts, settings, etc.) |
| `common` | État global de l'application, modèles de données, communication avec Warp |
| `icons` | Bibliothèque d'icônes SVG |
| `extensions` | Système de plugins/extensions |
| `native_extensions/emoji_selector` | Sélecteur d'emojis natif |

Les dépendances externes majeures incluent : Dioxus (UI), Warp/IPFS (réseau P2P et stockage), pulldown-cmark (Markdown), regex, arboard (presse-papiers), et uuid.

### 2.3 Inventaire des modules testables

| Module | Composants principaux | Testabilité | Risque métier |
|---|---|---|---|
| **Message** (`kit/src/components/message/`) | Formatage, Markdown, liens, emojis, rendu UI | Élevée — contient de nombreuses fonctions pures | Élevé — cœur de l'expérience utilisateur |
| **Chat** (`ui/src/layouts/chats/`) | Gestion des conversations, envoi, historique | Moyenne — dépendances réseau importantes | Élevé |
| **Friends** (`ui/src/layouts/friends/`) | Demandes d'amis, liste de contacts | Moyenne — dépend de MultiPass/Warp | Moyen |
| **Settings** (`ui/src/layouts/settings/`) | Configuration, profil, apparence | Moyenne | Faible |
| **File transfer** (`kit/src/components/embeds/`) | Transfert de fichiers, aperçus, téléchargement | Faible — I/O et réseau lourds | Moyen |
| **Audio/Video** (via warp-blink-wrtc) | Appels WebRTC | Faible — entièrement dépendant du runtime réseau | Moyen |
| **Common/State** (`common/src/state/`) | État global, réducteur, actions | Élevée — logique pure | Élevé |

---

## 3. Périmètre de test et priorisation

### 3.1 Approche de priorisation

Dans un contexte de ressources et de temps limités, il est nécessaire de prioriser les efforts de test. La sélection du périmètre s'appuie sur une analyse croisant trois critères :

- **Risque métier :** l'impact sur l'utilisateur final si le module est défaillant
- **Testabilité :** la facilité à écrire des tests automatisés isolés, sans dépendances lourdes (réseau, hardware)
- **Densité logique :** la quantité de logique métier contenue dans le module (parseurs, transformations, conditions)

### 3.2 Justification du périmètre sélectionné

Le module **message** (`kit/src/components/message/mod.rs`) a été retenu comme périmètre prioritaire de cette campagne de tests pour les raisons suivantes :

**1. C'est la fonctionnalité cœur de l'application.** Uplink est avant tout une application de messagerie. Le composant message est sollicité à chaque interaction : envoi, réception, affichage, édition, réaction. Un défaut dans ce module affecte directement et visiblement l'ensemble des utilisateurs. Il représente le point de contact le plus fréquent entre l'utilisateur et l'application.

**2. Le module présente une testabilité excellente.** Contrairement aux modules réseau (Warp, IPFS, WebRTC) qui nécessitent une infrastructure P2P complète pour être testés, le module message contient un grand nombre de fonctions pures — `format_text`, `markdown`, `replace_emojis`, `wrap_links_with_a_tags`, `is_only_emojis`, `process_string` — qui acceptent des entrées et retournent des sorties déterministes, sans effets de bord. Cela permet d'écrire des tests unitaires fiables, rapides et reproductibles.

**3. La surface de code est riche en logique de transformation.** Le fichier `mod.rs` (1085 lignes) contient du parsing Markdown, de la détection de liens par regex, de l'échappement HTML, de la conversion d'émoticônes, et de la gestion des mentions. Cette densité logique multiplie les cas limites et les possibilités d'erreur, ce qui justifie un effort de test approfondi.

**4. Les enjeux de sécurité sont concrets.** Le module manipule du contenu utilisateur qui est ensuite injecté dans le DOM via `dangerous_inner_html`. Un échappement HTML défaillant ouvre la porte à des attaques XSS (Cross-Site Scripting). Tester rigoureusement les fonctions d'échappement est donc une nécessité de sécurité.

**5. Le retour sur investissement est maximal.** En testant ce module en profondeur, on obtient une couverture de la fonctionnalité la plus utilisée de l'application tout en construisant une suite de tests réutilisable. Les tests créés ici pourront servir de socle de régression pour toutes les évolutions futures du système de messagerie.

> **Note :** Nous reconnaissons qu'une approche plus conventionnelle commencerait par tester les couches les plus critiques du point de vue système — typiquement la gestion de l'état (`common/state`) ou la couche réseau. Cependant, la couche réseau d'Uplink repose entièrement sur Warp/IPFS, qui sont des dépendances externes avec leur propre suite de tests. Tester ces couches reviendrait en grande partie à tester des bibliothèques tierces plutôt que la logique propre à Uplink. Le module message, en revanche, contient de la logique 100% propre au projet, ce qui maximise la pertinence des tests écrits.

### 3.3 Périmètre détaillé du module message

Le fichier `kit/src/components/message/mod.rs` contient les éléments suivants :

| Fonction / Struct | Rôle | Complexité |
|---|---|---|
| `format_text()` | Point d'entrée du formatage : échappe HTML, traite les mentions, appelle markdown ou replace_emojis | Élevée |
| `markdown()` | Convertit le texte en HTML via pulldown-cmark, gère les emojis, les blocs de code, et ignore les liens/images | Élevée |
| `wrap_links_with_a_tags()` | Détecte les URLs (http, www, mailto) et les encapsule dans des balises `<a>` | Moyenne |
| `LinkReplacer` (struct) | Implémente `Replacer` pour la transformation regex des liens | Moyenne |
| `replace_emojis()` | Remplace les émoticônes ASCII (`:)`, `;)`, etc.) par des emojis Unicode | Faible |
| `process_string()` | Traitement générique par mots avec une fonction de callback | Faible |
| `stack_processor()` | Mappe les tokens vers des emojis ou déséchappe le HTML | Faible |
| `is_only_emojis()` | Détecte si un texte ne contient que des emojis (pour affichage agrandi) | Moyenne |
| `Order` (enum) | Définit l'ordre d'affichage : First, Middle, Last | Faible |
| `ReactionAdapter` | Structure des réactions (emoji, compteur, self_reacted) | Faible |
| `Message` (composant UI) | Composant Dioxus principal pour le rendu d'un message | Élevée |
| `ChatText` (composant UI) | Sous-composant pour le rendu du texte formaté | Moyenne |
| `EditMsg` (composant UI) | Sous-composant pour l'édition de messages | Faible |
| `IdentityMessage` | Affiche un message de type « carte d'identité » d'un utilisateur | Élevée |

### 3.4 Éléments hors périmètre

Les éléments suivants ne sont **pas** couverts par cette campagne de tests :

- Le composant `IdentityMessage` (dépend entièrement de Warp et du réseau P2P)
- Les appels réseau et la communication IPFS
- Les modules audio/vidéo (WebRTC)
- Le système d'extensions
- Les modules UI autres que le composant message (settings, friends, etc.)

Ces modules pourront faire l'objet de campagnes de tests ultérieures, en suivant la même méthodologie.

---

## 4. Types de tests

### 4.1 Vue d'ensemble

La stratégie de test suit la pyramide de tests classique, avec une base solide de tests unitaires, complétée par des tests d'intégration et des tests E2E.

| Niveau | Quantité min. | Cible | Approche |
|---|---|---|---|
| Tests unitaires | 50+ | Fonctions pures : `format_text`, `markdown`, `replace_emojis`, `wrap_links_with_a_tags`, `is_only_emojis`, `process_string`, `stack_processor` | Automatique (`cargo test`) |
| Tests d'intégration | 5+ | Interaction entre `format_text`, `markdown` et `wrap_links_with_a_tags` ; chaînes de formatage complètes | Automatique (`cargo test`) |
| Tests E2E manuels | 1+ | Envoi et réception d'un message avec formatage markdown dans l'interface complète | Manuel |
| Tests E2E automatisés | 1+ | Scénario d'envoi de message via l'interface | Automatique (Selenium / Playwright / autre) |

### 4.2 Tests unitaires

Les tests unitaires couvrent **intégralement** les fonctions de traitement de texte du module message. Chaque fonction est testée avec des scénarios nominaux, d'exception et limites. Les tests utilisent des données de test pertinentes, et les dépendances externes (State, Warp) sont mockées.

**Répartition prévue des tests unitaires :**

| Fonction | Nb tests | Scénarios clés |
|---|---|---|
| `format_text()` | 10–12 | Texte simple, markdown activé/désactivé, emojis activés/désactivés, échappement HTML, chaîne vide, mentions (avec mock State) |
| `markdown()` | 10–12 | Gras, italique, barré, code inline, bloc de code, listes, emojis dans markdown, texte sans markdown, caractères spéciaux, big emoji |
| `replace_emojis()` | 6–8 | Chaque émoticône ASCII (`:)`, `:(`, `;)`, `:D`, `xD`, `:p`, etc.), texte mixte, aucun emoji |
| `process_string()` | 4–5 | Mots séparés par espaces, chaîne vide, un seul mot, caractères spéciaux |
| `stack_processor()` | 6–8 | Chaque émoticône, mode unescape_html, mode sans emoji, entrée inconnue |
| `wrap_links_with_a_tags()` | 6–8 | URL http, https, www, mailto, texte sans lien, URLs multiples, URL avec parenthèses |
| `is_only_emojis()` | 6–8 | Emoji unique, multiples emojis, texte + emoji, chaîne vide, caractères spéciaux, émojis composés (ZWJ) |
| `Order` (enum) | 3 | Display trait : First, Middle, Last |
| `HTML_ESCAPES` | 3–5 | Vérification de chaque paire d'échappement dans format_text |

### 4.3 Tests d'intégration

Les tests d'intégration vérifient l'interaction entre plusieurs fonctions du module. Ils couvrent **partiellement** les fonctionnalités en combinant formatage, détection de liens et rendu.

| ID | Scénario d'intégration | Fonctions impliquées |
|---|---|---|
| INT-01 | Formatage complet d'un message avec markdown + liens + emojis | `format_text` → `markdown` → `wrap_links_with_a_tags` |
| INT-02 | Message avec échappement HTML suivi de markdown et détection de liens | `format_text` (HTML escape) → `markdown` → `wrap_links_with_a_tags` |
| INT-03 | Message contenant uniquement des emojis doit avoir la classe big-emoji | `format_text` → `replace_emojis` → `is_only_emojis` |
| INT-04 | Message avec liens mailto et URL mixtes dans du texte markdown | `markdown` → `wrap_links_with_a_tags` (avec mailto) |
| INT-05 | Chaîne contenant des caractères HTML malveillants suivie de formatage complet | `format_text` (XSS prevention) → `markdown` |
| INT-06 | Message avec blocs de code contenant des emojis ASCII (ne doivent pas être remplacés dans le code) | `markdown` (code block) + `stack_processor` |

### 4.4 Tests E2E

**Test E2E manuel (E2E-M01) :**

Scénario : Un utilisateur envoie un message contenant du texte en gras, un lien, et un emoji. L'autre utilisateur vérifie que le message s'affiche correctement avec le formatage appliqué.

| Étape | Action | Résultat attendu |
|---|---|---|
| 1 | Lancer Uplink et se connecter | L'application s'ouvre, l'utilisateur est connecté |
| 2 | Ouvrir une conversation existante | La conversation s'affiche avec l'historique |
| 3 | Saisir : `**Bonjour** ! Regarde https://example.com :)` | Le texte est saisi dans le champ |
| 4 | Envoyer le message | Le message apparaît dans la conversation |
| 5 | Vérifier le rendu | « Bonjour » est en gras, le lien est cliquable, `:)` est remplacé par 🙂 |

**Test E2E automatisé (E2E-A01) :**

Scénario automatisé utilisant un framework d'automatisation (ex. : test d'interface Dioxus ou Selenium sur la version web). Le test vérifie l'envoi d'un message et la présence du texte formaté dans le DOM.

---

## 5. Infrastructure de test

### 5.1 Environnement technique

| Composant | Technologie | Détail |
|---|---|---|
| Langage | Rust | Le code source et les tests sont écrits en Rust |
| Framework de test unitaire | `cargo test` (built-in) | Framework de test intégré à Rust avec le module `#[cfg(test)]` |
| Framework de test d'intégration | `cargo test` (tests/ directory) | Tests d'intégration dans le répertoire `tests/` du crate kit |
| Framework E2E automatisé | Au choix : Selenium, Playwright, ou tests Dioxus natifs | Outil libre selon la configuration du projet |
| Mocking | `mockall` ou implémentation manuelle | Pour isoler les dépendances (State, Warp, etc.) |
| Couverture de code | `cargo-tarpaulin` ou `llvm-cov` | Génération de rapports de couverture |
| CI/CD | GitHub Actions | Workflows déjà présents dans `.github/workflows` |
| OS | Linux / macOS / Windows | Multi-plateforme, tests exécutés sur l'OS du développeur |

### 5.2 Données de test

Les tests utilisent trois types de données de test :

| Type | Usage | Exemple |
|---|---|---|
| Stub | Remplacer les dépendances externes par des valeurs fixes | Un State stub retourné avec des participants prédéfinis pour tester les mentions |
| Fake | Implémentations simplifiées de composants complexes | Un faux système de fichiers pour tester les pièces jointes |
| Mock | Vérification des interactions entre composants | Mock de l'EventHandler pour vérifier que `on_edit` est appelé lors de l'édition |

### 5.3 Exécution des tests

Commandes pour exécuter les différents types de tests :

```bash
# Tests unitaires
cargo test --package kit --lib components::message

# Tests d'intégration
cargo test --package kit --test integration_message

# Couverture de code
cargo tarpaulin --packages kit --out Html
```

---

## 6. Organisation des tests

### 6.1 Rôles et responsabilités

| Rôle | Responsabilités |
|---|---|
| Développeur/Testeur | Rédaction et exécution des tests unitaires et d'intégration, correction des défauts trouvés, mise à jour de la documentation |
| Testeur E2E | Conception et exécution des tests E2E manuels et automatisés, rédaction des rapports de test |
| Responsable qualité | Revue du concept de test, validation des rapports de couverture, classification des défauts |

### 6.2 Structure des fichiers de test

Les tests sont organisés selon la convention Rust :

```
kit/
├── src/
│   └── components/
│       └── message/
│           └── mod.rs              (tests unitaires inline #[cfg(test)])
├── tests/
│   └── integration_message.rs      (tests d'intégration)
└── e2e/
    ├── manual_e2e_message.md       (procédure E2E manuelle)
    └── auto_e2e_message.rs         (test E2E automatisé)
```

### 6.3 Processus de test

Le processus suit une approche itérative :

- **Étape 1 — Analyse :** Lecture et compréhension du code source de mod.rs
- **Étape 2 — Conception :** Rédaction des cas de test (scénarios nominaux, exception, limites)
- **Étape 3 — Implémentation :** Écriture des tests unitaires (50+), intégration (5+), E2E
- **Étape 4 — Exécution :** Lancement des tests, mesure de la couverture
- **Étape 5 — Rapport :** Documentation des résultats, classification des défauts
- **Étape 6 — Correction :** Correction des défauts identifiés et re-test

---

## 7. Plan de test

### 7.1 Planning prévisionnel

| Phase | Durée | Début | Fin | Livrable |
|---|---|---|---|---|
| Analyse du code + rédaction du concept | 1 semaine | 24.03.2026 | 28.03.2026 | Concept de test (ce document) |
| Rédaction des tests unitaires | 2 semaines | 31.03.2026 | 11.04.2026 | 50+ tests unitaires dans mod.rs |
| Rédaction des tests d'intégration | 1 semaine | 14.04.2026 | 18.04.2026 | 5+ tests d'intégration |
| Rédaction des tests E2E | 1 semaine | 21.04.2026 | 25.04.2026 | 1 test manuel + 1 test automatisé |
| Exécution, rapport + vidéo | 1 semaine | 28.04.2026 | 05.05.2026 | Rapport de couverture + vidéo |
| Remise finale | — | — | 08.05.2026 | Tous les livrables |

### 7.2 Livrables

- Concept de test (PDF) — ce document
- Code source des tests (dans l'archive ZIP avec l'application)
- Fichiers README pour l'exécution de chaque suite de tests
- Rapport de couverture de code
- Vidéo de démonstration (max 20 min, format mp4)

---

## 8. Classification des défauts

### 8.1 Niveaux de sévérité

| ID | Sévérité | Description | Exemple |
|---|---|---|---|
| S1 | Critique | Le module crash ou provoque une perte de données. L'application est inutilisable. | Panic dans `format_text()` avec une entrée spécifique |
| S2 | Élevée | Fonctionnalité majeure défaillante, pas de contournement simple. | Le markdown ne fonctionne plus du tout |
| S3 | Moyenne | Fonctionnalité partiellement défaillante, contournement possible. | Un type de lien (mailto) n'est pas détecté |
| S4 | Faible | Défaut mineur, esthétique ou documentation. | Emoji `:/` affiche le mauvais caractère |

### 8.2 Niveaux de priorité

| ID | Priorité | Description |
|---|---|---|
| P1 | Immédiate | Doit être corrigé avant la livraison. Bloque les tests. |
| P2 | Élevée | Doit être corrigé dès que possible, avant la remise finale. |
| P3 | Normale | À corriger si le temps le permet. |
| P4 | Basse | Amélioration souhaitée, non bloquante. |

### 8.3 Cycle de vie d'un défaut

Chaque défaut suit le cycle de vie suivant : **Nouveau** → **Assigné** → **En cours** → **Résolu** → **Vérifié** → **Fermé**. Si la vérification échoue, le défaut retourne à l'état « En cours ». Les défauts sont suivis dans un document dédié ou via les Issues GitHub du dépôt.

---

## 9. Critères d'entrée et de sortie

### 9.1 Critères d'entrée

Les conditions suivantes doivent être remplies avant de commencer les tests :

- Le code source de mod.rs est disponible et compile sans erreur
- L'environnement Rust et cargo sont installés et fonctionnels
- Les dépendances du projet (pulldown-cmark, regex, unic-emoji-char, etc.) sont résolues
- Le concept de test est validé
- Le MVP de l'application est fonctionnel

### 9.2 Critères de sortie

Les tests sont considérés terminés lorsque :

- Tous les 50+ tests unitaires passent avec succès
- Tous les 5+ tests d'intégration passent avec succès
- Les tests E2E (1 manuel + 1 automatisé) ont été exécutés et documentés
- La couverture de code atteint ≥ 80% sur les fonctions cibles
- Aucun défaut de sévérité S1 ou S2 n'est ouvert
- Les rapports et la documentation sont complets

### 9.3 Critères de suspension

Les tests sont suspendus si : le code ne compile plus, une dépendance critique est cassée, ou un défaut S1 empêche l'exécution des autres tests. La reprise se fait dès que le problème est résolu.

---

## 10. Glossaire

| Terme / Acronyme | Définition |
|---|---|
| E2E | End-to-End — test de bout en bout simulant un parcours utilisateur complet |
| MVP | Minimum Viable Product — version minimale fonctionnelle de l'application |
| P2P | Peer-to-Peer — architecture réseau décentralisée |
| XSS | Cross-Site Scripting — vulnérabilité d'injection de code malveillant |
| Dioxus | Framework UI en Rust utilisé par Uplink |
| Warp | Couche de communication P2P utilisée par Uplink |
| IPFS | InterPlanetary File System — système de fichiers distribué |
| DID | Decentralized Identifier — identifiant décentralisé |
| Mock | Objet simulant le comportement d'un composant réel pour les tests |
| Stub | Valeur de retour prédéfinie remplaçant une dépendance dans un test |
| Fake | Implémentation simplifiée d'un composant pour les tests |
| CI/CD | Continuous Integration / Continuous Deployment |
| ZWJ | Zero Width Joiner — caractère Unicode pour combiner des emojis |
| WebRTC | Web Real-Time Communication — protocole pour les appels audio/vidéo |
| DOM | Document Object Model — représentation en arbre du contenu HTML |
