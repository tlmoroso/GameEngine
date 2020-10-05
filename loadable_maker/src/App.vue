<template>
  <div id="app">
    <Maker></Maker>
  </div>
</template>

<script>
import Maker from "@/components/Maker";

export default {
  name: 'App',
  components: {
    Maker,
  }
}
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
