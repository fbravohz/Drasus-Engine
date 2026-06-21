// Test de integración: confirma que el crate nautilus-model compila correctamente
// y que al menos un tipo NT es accesible a través de la capa anticorrupción stub.
// Este test no ejercita lógica de negocio — solo verifica que la dependencia
// resuelve, los tipos existen en los paths declarados y el workspace compila limpio.

#[test]
fn nautilus_crates_compile_and_basic_type_is_accessible() {
    // Importa el tipo a través del stub; si el crate no compiló o el path cambió,
    // este test falla en tiempo de compilación — antes de ejecutarse.
    use nautilus_compat::stub::AccountType;

    // TypeId::of::<T>() devuelve el identificador único del tipo en tiempo de ejecución.
    // No necesitamos instanciar el tipo; con obtener su TypeId es suficiente para
    // confirmar que el tipo es conocido por el compilador y está enlazado correctamente.
    let _ = std::any::TypeId::of::<AccountType>();
}
