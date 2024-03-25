git add .
git commit -m "Deploy"
git tag --delete latest
git tag $1
git tag latest
git push origin HEAD
git push origin HEAD --tags --force