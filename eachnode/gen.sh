#!/bin/sh

set -eu

CURRENT_DIR="$(pwd)"
: ${SCRIPT_DIR:="$(dirname "$0")"}
EXE="${SCRIPT_DIR}/../target/release/fbx_objects_depviz"
TEMPLATE="${SCRIPT_DIR}/template.json"

: ${SHOW_IMPLICIT_NODES:=false}

if [ $# -lt 1 ] ; then
    echo "Usage: gen.sh <FBX_FILE>" >&2
    exit 2
fi

FBX="$1"
FBX_STEM="$(basename "$FBX" .json)"
RESULT_DIR="${CURRENT_DIR}/${FBX_STEM}"
mkdir -p "${RESULT_DIR}"


pushd "${SCRIPT_DIR}/.." >/dev/null
cargo build --release
popd >/dev/null


filter_sub() {
    RESULT_DIR_LOCAL="$1"
    mkdir -p "${RESULT_DIR_LOCAL}"
    CLASS="$2"
    SUBCLASS_NAME="${3:-}"
    if [ "x${SUBCLASS_NAME}" == "x" ] ; then
        SUBCLASS_PAT='.*'
    else
        SUBCLASS_PAT="^${SUBCLASS_NAME}\$"
    fi
    if [ "x${4:-}" == "xtrue" ] ; then
        SHOW_IMP_VAL="true"
    else
        SHOW_IMP_VAL="false"
    fi
    echo -n "Processing ${CLASS}::${SUBCLASS_NAME}..."
    DOT_OUT="${RESULT_DIR_LOCAL}/${CLASS}_${SUBCLASS_NAME}.dot"
    SVG_OUT="${RESULT_DIR_LOCAL}/${CLASS}_${SUBCLASS_NAME}.svg"
    PNG_OUT="${RESULT_DIR_LOCAL}/${CLASS}_${SUBCLASS_NAME}.png"
    TEMP_TEMPLATE="${SCRIPT_DIR}/temp_template_$$.json"
    sed \
        -e "s/<<class>>/^${CLASS}\$/" \
        -e "s/<<subclass>>/${SUBCLASS_PAT}/" \
        -e "s/<<show_implicit_nodes>>/${SHOW_IMP_VAL}/" \
        "$TEMPLATE" \
        >"${TEMP_TEMPLATE}"
    "$EXE" "$FBX" --output="$DOT_OUT" --filter="$TEMP_TEMPLATE"
    rm "${TEMP_TEMPLATE}"
    dot -Tsvg "$DOT_OUT" >"$SVG_OUT"
    dot -Tpng "$DOT_OUT" >"$PNG_OUT"
    echo " done."
}

filter() {
    echo "Without anonymous node"
    filter_sub "${RESULT_DIR}/explicit" "${1:-}" "${2:-}" "false"
    echo "With anonymous node"
    filter_sub "${RESULT_DIR}/with_anonymous" "${1:-}" "${2:-}" "true"
}

filter 'NodeAttribute'  'Null'
filter 'NodeAttribute'  'LimbNode'
filter 'Geometry'       'Mesh'
filter 'Geometry'       'Shape'
filter 'Model'          'Null'
filter 'Model'          'LimbNode'
filter 'Model'          'Mesh'
filter 'Pose'           'BindPose'
filter 'Material'
filter 'Deformer'       'Skin'
# Node name of `SubDeformer` is `Deformer`.
filter 'SubDeformer'    'Cluster'
filter 'Deformer'       'BlendShape'
filter 'SubDeformer'    'BlendShapeChannel'
filter 'Video'          'Clip'
filter 'Texture'
# Node name of `DisplayLayer` is `CollectionExclusive`.
filter 'DisplayLayer'   'DisplayLayer'
