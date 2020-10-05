<template>
    <div class="main">
        <h1>{{ title }}</h1>
        <b-dropdown v-bind:text="current_loadable">
            <b-dropdown-item v-on:click="on_loadable_select(loadable)" v-for="(loadable, index) in loadables" v-bind:key="index"> {{ loadable.name }} </b-dropdown-item>
        </b-dropdown>
    </div>
</template>

<script>
    export default {
        name: "Maker",
        props: {
            title: {
                type: String,
                default: "Loadable Maker"
            }
        },
        data: function() {
            return {
                current_loadable: "none",
                loadables: [
                    {
                        name: "Scene Stack",
                        id: "scene_stack"
                    },
                    {
                        name: "Scene",
                        id: "scene",
                    },
                    {
                        name: "Entity",
                        id: "entity"
                    },
                    {
                        name: "Animation",
                        id: "animation",
                    },
                    {
                        name: "Mesh Graphic",
                        id: "mesh_graphic"
                    },
                    {
                        name: "Player Control",
                        id: "player_control"
                    },
                    {
                        name: "Position",
                        id: "position"
                    },
                    {
                        name: "Text Display",
                        id: "text_display"
                    }
                ]
            };
        },
        methods: {
            on_loadable_select(loadable) {
                this.current_loadable = loadable.name;
            }
        },
        computed: {

        }
    }
</script>

<style scoped>

</style>