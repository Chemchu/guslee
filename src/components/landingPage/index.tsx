import { motion } from "framer-motion";
import Image from "next/image";
import { useState } from "react";
import Card from "../card";

const LandingPage = () => {
    const descripciones: string[] = ['ingeniero de software', 'guitarrita', 'brasileño', 'fullstack developer', 'panadero', 'gracioso', 'carismático'];

    return (
        <FondoImagen descripciones={descripciones} />

        //<FondoVideo />

    );
}

const FondoVideo = () => {
    return (
        <motion.div className="items-center font-sans overflow-hidden" >
            <video autoPlay loop muted className='w-full h-full object-cover fixed -z-10'>
                <source src={'/video/tv.mp4'} type="video/mp4" />
            </video>
            <div className="text-white p-4 text-left items-end">
                <div className="flex text-3xl md:text-5xl lg:text-7xl">
                    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 0 } }}
                        className="font-outfit mr-2 md:mr-4">
                        ¡Hola!
                    </motion.div>
                    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 1 } }}
                        className="font-outfit-thin">
                        Me llamo Gustavo Lee
                    </motion.div>
                </div>
                <div className="text-lg md:text-3xl lg:text-5xl my-4">
                    <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 1.8 } }}
                        className="font-outfit-thin mr-2 md:mr-4">
                        Soy ingeniero de software
                    </motion.div>
                </div>

            </div>
        </motion.div >
    );
}

const FondoImagen = (props: { descripciones: string[] }) => {
    const [currentDescription, setCurrentDescription] = useState<string>('informático');

    const ChangeDescription = () => {
        if (props.descripciones.length <= 0) { throw new Error('Lista de descripciones vacía') }

        const d = props.descripciones.shift();
        if (!d) { return; }

        props.descripciones.push(d);
        setCurrentDescription(d);
    }

    return (
        <main className="bg-main bg-cover bg-no-repeat bg-center font-outfit-light w-screen h-screen overflow-hidden cursor-default">
            <div className="text-white m-4 text-left items-end">
                <div className="flex text-xl sm:text-4xl xl:text-6xl">
                    <motion.div initial={{ opacity: 0, x: '100vw' }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 0 } }}
                        className="font-outfit mr-2 md:mr-4">
                        ¡Hola!
                    </motion.div>
                    <motion.div initial={{ opacity: 0, x: '100vw' }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 1 } }}
                        className="font-outfit-thin">
                        Me llamo Gustavo Lee
                    </motion.div>
                </div>
                <div className="text-lg sm:text-3xl xl:text-5xl my-4">
                    <motion.div initial={{ opacity: 0, x: '100vw' }} animate={{ opacity: 1, x: 0, transition: { duration: 1.5, ease: [0.87, 0, 0.13, 1], delay: 1.8 } }}
                        className="flex font-outfit-thin mr-2 md:mr-4">
                        <div className="pr-2">
                            Soy
                        </div>
                        <div className="transform duration-200 hover:scale-105 cursor-pointer hover:animate-bounce"
                            onClick={ChangeDescription}>
                            {currentDescription}
                        </div>
                    </motion.div>
                </div>
            </div>
            <div className="w-full h-full p-4">
                <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1, transition: { duration: 1 } }}
                    className="flex w-full h-4/5 rounded-xl bg-orange-400 shadow">
                    <Card />

                </motion.div>
            </div>
        </main >
    );
}

export default LandingPage;