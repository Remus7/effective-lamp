<script setup lang="ts">
import { ref } from "vue"
import { invoke } from "@tauri-apps/api/tauri"
import { listen } from '@tauri-apps/api/event'

const displayedTemp = ref("");
const displayedCity = ref("");
const displayedIcon = ref("");
const displayedCountry = ref("");
const visible = ref(false);
const inputCity = ref("")
const favoriteCities = ref([]);

const debugMsg = ref("");
const lastCoords = ref([0, 0]);
const refreshing = ref(false);

const pageOpen = ref(false);

const currentLocation = ref([0, 0])
navigator.geolocation.getCurrentPosition(async (pos) => {
  currentLocation.value[0] = pos.coords.latitude;  
  currentLocation.value[1] = pos.coords.longitude;  
});

invoke("init_process", {time: 30});

const unlisten = listen('getlocation', () => {
  if(pageOpen.value == true){
    getWeather(lastCoords.value as [number, number]);
    refreshing.value = true;
  }
})

async function getWeather(coord: [number, number]){
  try{
    displayedTemp.value = await invoke("get_temperature", {latitude: coord[0], longitude: coord[1]});
    displayedIcon.value = await invoke("get_image", {latitude: coord[0], longitude: coord[1]});

    let location: [string, string] = await invoke("get_city", {latitude: coord[0], longitude: coord[1]});
    displayedCity.value = location[0];
    displayedCountry.value = location[1];
    lastCoords.value = coord;

  } catch(e){
    debugMsg.value = e as string;
  } 

  refreshing.value = false;
  visible.value = true;
}

async function getCityInformation() {
  visible.value = false;
  pageOpen.value = true;

  try{
    let coord = await invoke("get_location", {city: inputCity.value, state: "", country: ""}) as [number, number];
    getWeather(coord);

  } catch(e){
    debugMsg.value = e as string;
  } 
}

async function getCurrentLocationInformation() {
  visible.value = false;
  pageOpen.value = true;

  try{
    getWeather(currentLocation.value as [number, number]);
  } catch(e){
    debugMsg.value = e as string;
  } 

  /* TO-DO (9)
    Be creative! Expand upon your app and add whatever functionalities you want.
    A few ideas could be: holiday suggestions based on favorite city forecasts, dynamic
    background color based on the weather, storing the weather history and displaying a chart of
    the temperature changes, dynamic weather timeline, weather fetching in parallel for multiple cities etc.
  */
}
</script>

<template>
  <button @click = "getCityInformation">Check City Weather</button>
  <input v-model="inputCity" placeholder="City Name" class="command_button">
  <br>
  <br>
  <button @click = "getCurrentLocationInformation" class="command_button">Check Current Location Weather</button>
  <br>
  <br>
  <div class="container">
    <div v-if="visible" class="text-background">
      <h1>{{ displayedCity }}</h1>
      <h3>{{ displayedCountry }}</h3>
      <h2>{{ displayedTemp }}</h2>
      <p>{{ debugMsg }}</p>
      <img :src="displayedIcon" alt="Weather icon">

      <br>
      <br>
      <!--<img v-if="refreshing" src="../images/refresh_icon.png" alt="Refresh icon">-->
    </div>
    <h2 class="text-background" v-else-if="pageOpen">Loading information...</h2>
    <!-- TO-DO (7)
        Make it so that the user can favorite a certain city that they searched.
        The user can then see all of their favorited cities as a list on their home screen.
        When selecting one of their favorite cities, they will be taken to a new page
        where they can see the details of that city.
        You can also make a self-updating temperature display for this page as well.
    -->
    <button v-for="city in favoriteCities">
      {{ city }}
    </button>
    <!-- TO-DO (8)
        Make it so that the user's favorite cities are persistent even once the app has been closed
        and reopened again.
        (HINT: write the array of cities to a file)
    -->
  </div>
</template>

<style>
  /* Styles for the body */
  body {
    margin: 0;
    padding: 0;
    font-family: Arial, sans-serif;
    background: url('background-image.jpg') no-repeat center center fixed;
    background-size: cover;
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
  }

  .container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    position: relative;
  }

  .centered-icon {
    width: 50px;
    height: 50px;
  }

  :root {
    background: url(../images/weather_background.png);
    background-repeat: no-repeat;
    background-size: cover;
    background-position: center;
  }

  .text-background {
    min-width: 400px;
    background-color: rgba(0, 0, 0, 0.35);
    padding: 10px;
    border-radius: 5px;
    color: white;
    margin-top: 10px;
  }
</style>
