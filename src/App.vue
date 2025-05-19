<template>
  <div class="flex h-screen w-screen flex-col bg-stone-200">
    <TitleBar />

    <div class="flex min-h-0 flex-1 overflow-auto">
      <Sidebar />

      <div class="h-full min-w-0 flex-1 overflow-auto pt-1 pr-2 pb-2 pl-0">
        <RouterView />
      </div>
    </div>

    <BottomBar />
  </div>
</template>
