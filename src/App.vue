<script setup lang="ts">
import AppSidebar from './components/AppSidebar.vue';
import { SidebarProvider, SidebarTrigger } from './components/ui/sidebar';
import { Toaster } from './components/ui/sonner';
import { TooltipProvider } from './components/ui/tooltip';
</script>

<template>
  <TooltipProvider :delay-duration="500">
    <SidebarProvider>
      <div class="w-screen h-screen flex">
        <AppSidebar />

        <main class="flex-1">
          <SidebarTrigger />
          <RouterView />
        </main>
      </div>
    </SidebarProvider>

    <Toaster />
  </TooltipProvider>
</template>
