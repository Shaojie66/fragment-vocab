#!/bin/bash

PROJECT_ID="PVT_kwHOBGUCYs4BRduJ"
STATUS_FIELD_ID="PVTSSF_lAHOBGUCYs4BRduJzg_SOuQ"
TODO_OPTION_ID="f75ad846"

ISSUES=(
  "I_kwDORkeKJ87x-TlC"
  "I_kwDORkeKJ87x-T0A"
  "I_kwDORkeKJ87x-T_d"
  "I_kwDORkeKJ87x-UPE"
  "I_kwDORkeKJ87x-UZr"
  "I_kwDORkeKJ87x-Ulc"
  "I_kwDORkeKJ87x-UzM"
  "I_kwDORkeKJ87x-U-4"
  "I_kwDORkeKJ87x-VKs"
  "I_kwDORkeKJ87x-VYe"
  "I_kwDORkeKJ87x-Vj6"
  "I_kwDORkeKJ87x-V3H"
  "I_kwDORkeKJ87x-WFS"
  "I_kwDORkeKJ87x-WUY"
  "I_kwDORkeKJ87x-Wl0"
  "I_kwDORkeKJ87x-Wxt"
  "I_kwDORkeKJ87x-W-J"
  "I_kwDORkeKJ87x-XKm"
)

for ISSUE_ID in "${ISSUES[@]}"; do
  echo "Adding issue $ISSUE_ID to project..."
  
  ITEM_ID=$(gh api graphql -f query="
    mutation {
      addProjectV2ItemById(input: {
        projectId: \"$PROJECT_ID\"
        contentId: \"$ISSUE_ID\"
      }) {
        item {
          id
        }
      }
    }
  " --jq '.data.addProjectV2ItemById.item.id')
  
  if [ -n "$ITEM_ID" ]; then
    echo "Setting status to Todo for item $ITEM_ID..."
    gh api graphql -f query="
      mutation {
        updateProjectV2ItemFieldValue(input: {
          projectId: \"$PROJECT_ID\"
          itemId: \"$ITEM_ID\"
          fieldId: \"$STATUS_FIELD_ID\"
          value: {
            singleSelectOptionId: \"$TODO_OPTION_ID\"
          }
        }) {
          projectV2Item {
            id
          }
        }
      }
    " > /dev/null
    echo "✓ Issue $ISSUE_ID added and set to Todo"
  else
    echo "✗ Failed to add issue $ISSUE_ID"
  fi
  
  sleep 0.5
done

echo "All issues added to project!"
