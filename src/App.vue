<script setup lang="ts">
const mockData = Array.from({ length: 5 }).map((_, index) => ({
  id: index + 1,
  method: 'GET',
  protocol: 'https',
  host: 'www.google.com',
  path: '/search',
  code: 200,
}));
</script>

<template>
  <main class="flex h-screen w-screen flex-col bg-stone-200">
    <TitleBar />

    <div class="flex flex-1 overflow-auto">
      <Sidebar />

      <div class="h-full min-w-0 flex-1 overflow-auto pt-1 pr-2 pb-2 pl-0">
        <Splitter class="h-full" :gutter-size="8">
          <SplitterPanel>
            <div class="bg-surface-0 h-full overflow-hidden rounded-lg">
              <DataTable :value="mockData" striped-rows size="small">
                <Column field="id" header="ID"></Column>
                <Column field="method" header="Method"></Column>
                <Column field="protocol" header="Protocol"></Column>
                <Column field="host" header="Host"></Column>
                <Column field="path" header="Path"></Column>
                <Column field="code" header="Code"></Column>
              </DataTable>
            </div>
          </SplitterPanel>

          <SplitterPanel>
            <div class="bg-surface-0 h-full overflow-hidden rounded-lg"></div>
          </SplitterPanel>
        </Splitter>
      </div>
    </div>

    <BottomBar />
  </main>
</template>
