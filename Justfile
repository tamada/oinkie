git_revision := `git rev-parse --short HEAD`
app_version := `awk -F'"' '/^\[package\]/{p=1} p && /^version *=/{print $2; exit}' Cargo.toml`
build_date := `date -u +%Y-%m-%dT%H:%M:%SZ`

image_quay   := 'quay.io/tama5'
image_github := 'ghcr.io/tamada'
container_image := image_quay

container-local:
    docker build \
        --build-arg GIT_REVISION={{git_revision}} \
        --build-arg BUILD_DATE={{build_date}} \
        --build-arg VERSION={{app_version}} \
        -t {{container_image}}/oinkie:latest \
        -t {{container_image}}/oinkie:light \
        -t {{container_image}}/oinkie:{{ app_version }} \
        -t {{container_image}}/oinkie:{{ app_version }}-light \
        -f containers/light/Containerfile \
        .
    docker build \
        --build-arg GIT_REVISION={{git_revision}} \
        --build-arg BUILD_DATE={{build_date}} \
        --build-arg VERSION={{ app_version }} \
        -t {{container_image}}/oinkie:full \
        -t {{container_image}}/oinkie:ghidra \
        -t {{container_image}}/oinkie:{{ app_version }}-full \
        -t {{container_image}}/oinkie:{{ app_version }}-ghidra \
        -f containers/full/Containerfile \
        .

container:
    docker buildx build --push \
        --platform linux/amd64,linux/arm64 \
        --build-arg GIT_REVISION={{git_revision}} \
        --build-arg BUILD_DATE={{build_date}} \
        --build-arg VERSION={{app_version}} \
        -t {{container_image}}/oinkie:latest \
        -t {{container_image}}/oinkie:light \
        -t {{container_image}}/oinkie:{{ app_version }} \
        -t {{container_image}}/oinkie:{{ app_version }}-light \
        -f containers/light/Containerfile \
        .

    docker buildx build --push \
        --platform linux/amd64,linux/arm64 \
        --build-arg GIT_REVISION={{git_revision}} \
        --build-arg BUILD_DATE={{build_date}} \
        --build-arg VERSION={{ app_version }} \
        -t {{container_image}}/oinkie:full \
        -t {{container_image}}/oinkie:ghidra \
        -t {{container_image}}/oinkie:{{ app_version }}-full \
        -t {{container_image}}/oinkie:{{ app_version }}-ghidra \
        -f containers/full/Containerfile \
        .
