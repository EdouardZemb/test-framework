# Quality Gates Configuration

Ce document définit les critères de qualité pour le pipeline CI/CD.

## Seuils de Passage

| Priorité | Taux Minimum | Action si Échec |
|----------|--------------|-----------------|
| **P0** (Critical) | 100% | ❌ Bloquer le merge |
| **P1** (High) | ≥ 95% | ❌ Bloquer le merge |
| **P2** (Medium) | ≥ 90% | ⚠️ Warning, review requis |
| **P3** (Low) | ≥ 80% | ℹ️ Info seulement |

## Burn-in Requirements

### Pull Requests

- **Iterations minimum :** 5
- **Condition :** Tests modifiés uniquement
- **Échec :** Bloque le merge

### Releases (main branch)

- **Iterations minimum :** 10
- **Condition :** Suite complète
- **Échec :** Bloque le déploiement

## Critères de Blocage CI

Le CI échoue automatiquement si :

1. ✅ **Tests P0/P1 échouent** — Aucune exception
2. ✅ **Burn-in échoue** — Flakiness détectée
3. ✅ **Taux global < 95%** — Régression de qualité
4. ⚠️ **Nouveaux tests sans couverture** — Warning (configurable)

## Notifications

### Slack (Optionnel)

Configurer dans `.github/workflows/test.yml` :

```yaml
- name: Notify Slack on failure
  if: failure()
  uses: slackapi/slack-github-action@v2
  with:
    webhook: ${{ secrets.SLACK_WEBHOOK }}
    payload: |
      {
        "text": "❌ E2E Tests Failed",
        "blocks": [
          {
            "type": "section",
            "text": {
              "type": "mrkdwn",
              "text": "*E2E Tests Failed* on `${{ github.ref_name }}`\n<${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}|View Run>"
            }
          }
        ]
      }
```

### Email

GitHub Actions envoie automatiquement des notifications d'échec aux auteurs de commits.

## Pre-Release Checklist

Avant chaque release, vérifier :

- [ ] Tous les tests P0/P1 passent
- [ ] Burn-in 10 iterations réussi
- [ ] Aucun test flaky détecté dans les 7 derniers jours
- [ ] Matrice de traçabilité à jour (workflow `/TR`)
- [ ] NFRs validés (workflow `/NR`)

## Métriques de Suivi

| Métrique | Cible | Alerte |
|----------|-------|--------|
| Taux de réussite global | ≥ 98% | < 95% |
| Tests flaky / semaine | ≤ 2 | > 5 |
| Durée CI moyenne | < 15 min | > 25 min |
| Couverture P0/P1 | 100% | < 100% |

---

*Généré par TEA CI Workflow*
